use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use voxforg_api::{create_app, AppState};
use voxforg_audio::WavEncoder;
use voxforg_core::models::AudioContainerFormat;
use voxforg_core::store::memory::MemoryStore;
use voxforg_engine::{
    AbTestRunner, AbTestScenario, EdgeTtsEngine, EngineRegistry, MockTtsEngine, OpenAiRouterEngine,
    PiperTtsEngine, Qwen3TtsEngine, SynthesisRequest,
};
use voxforg_hardware::HardwareProbe;

#[derive(Parser)]
#[command(name = "voxforg")]
#[command(author = "VoxForg Core Contributors")]
#[command(version = "0.1.0")]
#[command(about = "Unified Hardware-Adaptive Speech Synthesis Platform & Pipeline Engine", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the VoxForg HTTP/WS API server
    Serve(ServeArgs),

    /// Direct text-to-speech synthesis to audio file
    Synth(SynthArgs),

    /// Probe host hardware and display compute capabilities
    Hardware,

    /// List registered engines and available voices
    Engines,

    /// Run synthesis benchmark across available engines
    Bench(BenchArgs),

    /// Run automated A/B quality comparison between two voices/engines
    AbTest(AbTestArgs),

    /// Start Model Context Protocol (MCP) server over standard I/O
    Mcp,
}

#[derive(Args)]
struct ServeArgs {
    /// Network port to bind
    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    /// Network address to bind
    #[arg(long, default_value = "0.0.0.0")]
    host: String,

    /// Optional API key for authenticating incoming requests
    #[arg(long, env = "VOXFORG_API_KEY")]
    api_key: Option<String>,

    /// Data directory for pipeline graphs and models
    #[arg(long, default_value = "./data")]
    data_dir: PathBuf,

    /// Optional upstream model router endpoint URL (e.g. https://api.openai.com/v1 or http://localhost:8000/v1)
    #[arg(long, env = "VOXFORG_ROUTER_URL")]
    router_url: Option<String>,

    /// Optional upstream model router API key / Bearer token
    #[arg(long, env = "VOXFORG_ROUTER_API_KEY")]
    router_api_key: Option<String>,

    /// Optional upstream model router default TTS model (e.g. tts-1, kokoro)
    #[arg(long, env = "VOXFORG_ROUTER_MODEL")]
    router_model: Option<String>,
}

#[derive(Args)]
struct SynthArgs {
    /// Text to synthesize
    input: String,

    /// Target voice identifier (e.g. en-US-AriaNeural, mock-en-female, alloy, onyx)
    #[arg(short, long, default_value = "en-US-AriaNeural")]
    voice: String,

    /// Output audio file path (.wav)
    #[arg(short, long, default_value = "output.wav")]
    out: PathBuf,

    /// Speech rate multiplier (0.5 to 2.0)
    #[arg(short, long, default_value_t = 1.0)]
    speed: f32,

    /// Pitch adjustment in semitones (-12.0 to 12.0)
    #[arg(short, long, default_value_t = 0.0)]
    pitch: f32,

    /// Optional upstream model router endpoint URL
    #[arg(long, env = "VOXFORG_ROUTER_URL")]
    router_url: Option<String>,

    /// Optional upstream model router API key
    #[arg(long, env = "VOXFORG_ROUTER_API_KEY")]
    router_api_key: Option<String>,

    /// Optional upstream model router default TTS model
    #[arg(long, env = "VOXFORG_ROUTER_MODEL")]
    router_model: Option<String>,
}

#[derive(Args)]
struct BenchArgs {
    /// Number of iterations
    #[arg(short, long, default_value_t = 5)]
    iterations: usize,
}

#[derive(Args)]
struct AbTestArgs {
    /// Scenario name
    #[arg(short, long, default_value = "Automated CLI A/B Evaluation")]
    name: String,

    /// Benchmark test sentence
    #[arg(
        short,
        long,
        default_value = "The quick brown fox jumps over the lazy dog. Comparative synthesis benchmark."
    )]
    text: String,

    /// Voice for Variant A
    #[arg(long, default_value = "mock-en-female")]
    voice_a: String,

    /// Voice for Variant B
    #[arg(long, default_value = "en-US-AriaNeural")]
    voice_b: String,

    /// Speed for Variant A
    #[arg(long, default_value_t = 1.0)]
    speed_a: f32,

    /// Speed for Variant B
    #[arg(long, default_value_t = 1.0)]
    speed_b: f32,

    /// Pitch for Variant A
    #[arg(long, default_value_t = 0.0)]
    pitch_a: f32,

    /// Pitch for Variant B
    #[arg(long, default_value_t = 0.0)]
    pitch_b: f32,

    /// Number of evaluation iterations
    #[arg(short, long, default_value_t = 1)]
    iterations: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_target(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber).ok();

    let cli = Cli::parse();

    match cli.command {
        Commands::Serve(args) => run_serve(args).await,
        Commands::Synth(args) => run_synth(args).await,
        Commands::Hardware => run_hardware(),
        Commands::Engines => run_engines().await,
        Commands::Bench(args) => run_bench(args).await,
        Commands::AbTest(args) => run_ab_test(args).await,
        Commands::Mcp => run_mcp().await,
    }
}

async fn initialize_registry(
    router_url: Option<String>,
    router_api_key: Option<String>,
    router_model: Option<String>,
) -> Arc<EngineRegistry> {
    let registry = Arc::new(EngineRegistry::new());
    registry.register(Arc::new(EdgeTtsEngine::new())).await;
    registry.register(Arc::new(MockTtsEngine::default())).await;
    registry.register(Arc::new(Qwen3TtsEngine::default())).await;
    registry.register(Arc::new(PiperTtsEngine::new())).await;

    let r_url = router_url.or_else(|| std::env::var("VOXFORG_ROUTER_URL").ok());
    let r_key = router_api_key.or_else(|| std::env::var("VOXFORG_ROUTER_API_KEY").ok());
    let r_model = router_model.or_else(|| std::env::var("VOXFORG_ROUTER_MODEL").ok());

    let router_engine = if let Some(url) = r_url {
        OpenAiRouterEngine::new(url, r_key, r_model)
    } else {
        OpenAiRouterEngine::default_openai(r_key)
    };
    registry.register(Arc::new(router_engine)).await;

    registry
}

async fn run_serve(args: ServeArgs) -> Result<()> {
    info!("Starting VoxForg Workstation Server v0.1.0");

    if !args.data_dir.exists() {
        let _ = std::fs::create_dir_all(&args.data_dir);
    }

    let hardware = HardwareProbe::probe();
    info!("Host Architecture: {} / {}", hardware.os, hardware.arch);
    info!("Assigned Hardware Tier: {}", hardware.assigned_tier);
    info!(
        "CPU: {} ({} threads)",
        hardware.cpu_brand, hardware.cpu_logical_threads
    );

    let registry =
        initialize_registry(args.router_url, args.router_api_key, args.router_model).await;
    let store = Arc::new(MemoryStore::new());
    let state = AppState::new(registry, store, hardware, args.api_key.clone());

    let app = create_app(state);
    let addr: SocketAddr = format!("{}:{}", args.host, args.port).parse()?;

    info!("Listening for API requests on http://{}", addr);
    if args.api_key.is_some() {
        info!("Authentication enabled (Bearer token enforced)");
    } else {
        info!("Open mode (No API key enforced)");
    }

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    info!("VoxForg Server shut down cleanly.");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C interrupt. Draining server connections gracefully...");
        },
        _ = terminate => {
            info!("Received SIGTERM termination signal. Draining server connections gracefully...");
        },
    }
}

async fn run_synth(args: SynthArgs) -> Result<()> {
    let registry =
        initialize_registry(args.router_url, args.router_api_key, args.router_model).await;
    let (engine, voice) = registry.resolve_voice(&args.voice).await?;

    info!(
        "Synthesizing using engine: '{}' (Voice: '{}')",
        engine.name(),
        voice.name
    );
    let req = SynthesisRequest {
        text: args.input,
        voice_id: voice.id,
        speed: args.speed,
        pitch: args.pitch,
        format: AudioContainerFormat::Wav,
    };

    let start = Instant::now();
    let chunk = engine.synthesize(&req).await?;
    let duration = start.elapsed();

    let wav_bytes =
        WavEncoder::encode_pcm16_to_wav(&chunk.pcm_data, chunk.sample_rate, chunk.channels)?;

    if let Some(parent) = args.out.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    std::fs::write(&args.out, wav_bytes)?;

    info!(
        "Successfully wrote audio to {:?} (Generated {} samples in {:.2?})",
        args.out,
        chunk.pcm_data.len(),
        duration
    );
    Ok(())
}

fn run_hardware() -> Result<()> {
    let hw = HardwareProbe::probe();
    println!("=== VoxForg Hardware Analysis ===");
    println!("OS:                  {}", hw.os);
    println!("Architecture:        {}", hw.arch);
    println!("CPU:                 {}", hw.cpu_brand);
    println!("Physical Cores:      {}", hw.cpu_physical_cores);
    println!("Logical Threads:     {}", hw.cpu_logical_threads);
    println!("Total System RAM:    {} MB", hw.total_memory_mb);
    println!("Available RAM:       {} MB", hw.available_memory_mb);
    println!(
        "SIMD Capabilities:   AVX: {}, AVX2: {}, AVX-512: {}, NEON: {}",
        hw.simd.avx, hw.simd.avx2, hw.simd.avx512, hw.simd.neon
    );
    println!("GPU Adapters:        {}", hw.gpus.len());
    for (i, gpu) in hw.gpus.iter().enumerate() {
        println!(
            "  [{}] {} (VRAM: {} MB) [CUDA: {}, MPS: {}, DirectML: {}]",
            i, gpu.name, gpu.vram_mb, gpu.has_cuda, gpu.has_mps, gpu.has_directml
        );
    }
    println!("---------------------------------");
    println!("Assigned Tier:       {}", hw.assigned_tier);
    println!("Recommended Engines: {}", hw.recommended_engines.join(", "));
    Ok(())
}

async fn run_engines() -> Result<()> {
    let registry = initialize_registry(None, None, None).await;
    let voices = registry.list_all_voices().await?;

    println!("=== Registered Speech Engines ===");
    for engine_id in registry.list_engines().await {
        println!("- Engine ID: {}", engine_id);
    }

    println!("\n=== Available Voices (Total: {}) ===", voices.len());
    println!(
        "{:<25} {:<20} {:<10} {:<10} {:<10}",
        "Voice ID", "Name", "Engine", "Language", "Rate"
    );
    println!("{:-<75}", "");
    for v in voices {
        println!(
            "{:<25} {:<20} {:<10} {:<10} {}Hz",
            v.id, v.name, v.engine_id, v.language, v.sample_rate_hz
        );
    }
    Ok(())
}

async fn run_bench(args: BenchArgs) -> Result<()> {
    let registry = initialize_registry(None, None, None).await;
    let (engine, voice) = registry.resolve_voice("mock-en-female").await?;

    println!(
        "=== Benchmark: {} (Voice: {}) ===",
        engine.name(),
        voice.name
    );
    let test_text =
        "The quick brown fox jumps over the lazy dog. Speech synthesis benchmark test sentence.";

    let mut total_duration = std::time::Duration::ZERO;
    let iters = args.iterations.max(1);

    for i in 1..=iters {
        let req = SynthesisRequest {
            text: test_text.to_string(),
            voice_id: voice.id.clone(),
            speed: 1.0,
            pitch: 0.0,
            format: AudioContainerFormat::Wav,
        };

        let start = Instant::now();
        let chunk = engine.synthesize(&req).await?;
        let elapsed = start.elapsed();
        total_duration += elapsed;

        let audio_duration_sec = chunk.pcm_data.len() as f64 / chunk.sample_rate as f64;
        let rtf = elapsed.as_secs_f64() / audio_duration_sec;

        println!(
            "Iteration #{}: Elapsed: {:.2?}, Audio Duration: {:.2}s, RTF: {:.4}",
            i, elapsed, audio_duration_sec, rtf
        );
    }

    let avg = total_duration / (iters as u32);
    println!("---------------------------------");
    println!("Average Latency: {:.2?}", avg);
    Ok(())
}

async fn run_ab_test(args: AbTestArgs) -> Result<()> {
    let registry = initialize_registry(None, None, None).await;
    let runner = AbTestRunner::new(registry);

    let scenario = AbTestScenario {
        name: args.name,
        text: args.text,
        variant_a: SynthesisRequest {
            text: String::new(),
            voice_id: args.voice_a,
            speed: args.speed_a,
            pitch: args.pitch_a,
            format: AudioContainerFormat::Wav,
        },
        variant_b: SynthesisRequest {
            text: String::new(),
            voice_id: args.voice_b,
            speed: args.speed_b,
            pitch: args.pitch_b,
            format: AudioContainerFormat::Wav,
        },
    };

    println!("=== Running Automated A/B Evaluation ===");
    println!("Scenario: {}", scenario.name);
    println!("Test Text: \"{}\"", scenario.text);
    println!("Iterations: {}", args.iterations);
    println!("----------------------------------------");

    let iters = args.iterations.max(1);
    let mut last_comp = None;

    for i in 1..=iters {
        let comp = runner.run_comparison(&scenario).await?;
        if iters > 1 {
            println!(
                "[Run {}/{}] Latency A: {:.2}ms | Latency B: {:.2}ms | Faster: Variant {}",
                i, iters, comp.variant_a.latency_ms, comp.variant_b.latency_ms, comp.faster_variant
            );
        }
        last_comp = Some(comp);
    }

    let comp = last_comp.unwrap();

    println!(
        "Metric                  | Variant A ({:<18}) | Variant B ({:<18})",
        comp.variant_a.voice_id, comp.variant_b.voice_id
    );
    println!("{:-<75}", "");
    println!(
        "Engine                  | {:<30} | {:<30}",
        comp.variant_a.engine_id, comp.variant_b.engine_id
    );
    println!(
        "Latency                 | {:<28.2}ms | {:<28.2}ms",
        comp.variant_a.latency_ms, comp.variant_b.latency_ms
    );
    println!(
        "Audio Duration          | {:<29.2}s | {:<29.2}s",
        comp.variant_a.audio_duration_seconds, comp.variant_b.audio_duration_seconds
    );
    println!(
        "Real-Time Factor (RTF)  | {:<30.4} | {:<30.4}",
        comp.variant_a.realtime_factor, comp.variant_b.realtime_factor
    );
    println!(
        "Peak Amplitude (dBFS)   | {:<28.1}dB | {:<28.1}dB",
        comp.variant_a.metrics.peak_dbfs, comp.variant_b.metrics.peak_dbfs
    );
    println!(
        "RMS Loudness (dBFS)     | {:<28.1}dB | {:<28.1}dB",
        comp.variant_a.metrics.rms_dbfs, comp.variant_b.metrics.rms_dbfs
    );
    println!(
        "Clipping Samples Count  | {:<30} | {:<30}",
        comp.variant_a.metrics.clipping_samples_count,
        comp.variant_b.metrics.clipping_samples_count
    );
    println!("----------------------------------------");
    println!("Faster Variant:         Variant {}", comp.faster_variant);
    println!(
        "Recommended Variant:    Variant {}",
        comp.recommended_variant
    );
    println!("Summary:                {}", comp.summary);

    Ok(())
}

async fn run_mcp() -> Result<()> {
    let server = voxforg_mcp::McpServer::with_defaults();
    server.run_stdio().await?;
    Ok(())
}
