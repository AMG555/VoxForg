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
use voxforg_engine::{EdgeTtsEngine, EngineRegistry, MockTtsEngine, SynthesisRequest};
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
}

#[derive(Args)]
struct SynthArgs {
    /// Text to synthesize
    input: String,

    /// Target voice identifier (e.g. en-US-AriaNeural, mock-en-female)
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
}

#[derive(Args)]
struct BenchArgs {
    /// Number of iterations
    #[arg(short, long, default_value_t = 5)]
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
    }
}

async fn initialize_registry() -> Arc<EngineRegistry> {
    let registry = Arc::new(EngineRegistry::new());
    registry.register(Arc::new(EdgeTtsEngine::new())).await;
    registry.register(Arc::new(MockTtsEngine::default())).await;
    registry
}

async fn run_serve(args: ServeArgs) -> Result<()> {
    info!("Starting VoxForg Workstation Server v0.1.0");

    let hardware = HardwareProbe::probe();
    info!("Host Architecture: {} / {}", hardware.os, hardware.arch);
    info!("Assigned Hardware Tier: {}", hardware.assigned_tier);
    info!("CPU: {} ({} threads)", hardware.cpu_brand, hardware.cpu_logical_threads);

    let registry = initialize_registry().await;
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
    axum::serve(listener, app).await?;
    Ok(())
}

async fn run_synth(args: SynthArgs) -> Result<()> {
    let registry = initialize_registry().await;
    let (engine, voice) = registry.resolve_voice(&args.voice).await?;

    info!("Synthesizing using engine: '{}' (Voice: '{}')", engine.name(), voice.name);
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

    let wav_bytes = WavEncoder::encode_pcm16_to_wav(&chunk.pcm_data, chunk.sample_rate, chunk.channels)?;
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
    println!("SIMD Capabilities:   AVX: {}, AVX2: {}, AVX-512: {}, NEON: {}",
        hw.simd.avx, hw.simd.avx2, hw.simd.avx512, hw.simd.neon);
    println!("GPU Adapters:        {}", hw.gpus.len());
    for (i, gpu) in hw.gpus.iter().enumerate() {
        println!("  [{}] {} (VRAM: {} MB) [CUDA: {}, MPS: {}, DirectML: {}]",
            i, gpu.name, gpu.vram_mb, gpu.has_cuda, gpu.has_mps, gpu.has_directml);
    }
    println!("---------------------------------");
    println!("Assigned Tier:       {}", hw.assigned_tier);
    println!("Recommended Engines: {}", hw.recommended_engines.join(", "));
    Ok(())
}

async fn run_engines() -> Result<()> {
    let registry = initialize_registry().await;
    let voices = registry.list_all_voices().await?;

    println!("=== Registered Speech Engines ===");
    for engine_id in registry.list_engines().await {
        println!("- Engine ID: {}", engine_id);
    }

    println!("\n=== Available Voices (Total: {}) ===", voices.len());
    println!("{:<25} {:<20} {:<10} {:<10} {:<10}", "Voice ID", "Name", "Engine", "Language", "Rate");
    println!("{:-<75}", "");
    for v in voices {
        println!("{:<25} {:<20} {:<10} {:<10} {}Hz",
            v.id, v.name, v.engine_id, v.language, v.sample_rate_hz);
    }
    Ok(())
}

async fn run_bench(args: BenchArgs) -> Result<()> {
    let registry = initialize_registry().await;
    let (engine, voice) = registry.resolve_voice("mock-en-female").await?;

    println!("=== Benchmark: {} (Voice: {}) ===", engine.name(), voice.name);
    let test_text = "The quick brown fox jumps over the lazy dog. Speech synthesis benchmark test sentence.";

    let mut total_duration = std::time::Duration::ZERO;

    for i in 1..=args.iterations {
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

        println!("Iteration #{}: Elapsed: {:.2?}, Audio Duration: {:.2}s, RTF: {:.4}",
            i, elapsed, audio_duration_sec, rtf);
    }

    let avg = total_duration / (args.iterations as u32);
    println!("---------------------------------");
    println!("Average Latency: {:.2?}", avg);
    Ok(())
}
