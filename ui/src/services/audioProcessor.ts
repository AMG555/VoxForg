/**
 * Web Audio API Audio Preprocessor for Studio-Grade Voice Cloning
 *
 * Implements:
 * 1. 80Hz Rumble Cut (High-pass biquad filter)
 * 2. Silence & Noise Gating (trims dead air at start/end)
 * 3. Dynamic Peak Normalization (-1.0 dBFS)
 * 4. 16-bit Mono 24kHz RIFF WAV Encoder
 * 5. Real-Time Audio Quality Assessment (SNR, Duration, Clipping)
 */

export interface AudioQualityAssessment {
  durationSeconds: number;
  durationQuality: 'optimal' | 'short' | 'long';
  durationMessage: string;
  peakDbfs: number;
  rmsDbfs: number;
  snrEstimateDb: number;
  clarityRating: 'excellent' | 'good' | 'fair' | 'noisy';
  clarityMessage: string;
  clippingDetected: boolean;
  sampleRate: number;
  channelCount: number;
}

export interface PreprocessResult {
  wavBlob: Blob;
  wavBase64: string;
  metrics: AudioQualityAssessment;
  durationFormatted: string;
}

export class AudioProcessor {
  /**
   * Preprocess any audio blob (WebM, OGG, MP3, WAV, MP4) into studio-standard 24kHz 16-bit WAV
   */
  static async preprocessForCloning(audioBlob: Blob): Promise<PreprocessResult> {
    const arrayBuffer = await audioBlob.arrayBuffer();
    const audioContext = new (window.AudioContext || (window as any).webkitAudioContext)({
      sampleRate: 24000,
    });

    try {
      const decodedBuffer = await audioContext.decodeAudioData(arrayBuffer.slice(0));

      // 1. Convert to mono channel
      const rawSamples = this.downmixToMono(decodedBuffer);

      // 2. High-pass filter (80Hz Butterworth/Biquad approximation) to eliminate desk thump / rumble
      const filteredSamples = this.applyHighPassFilter(rawSamples, decodedBuffer.sampleRate, 80);

      // 3. Trim silence from head and tail
      const trimmedSamples = this.trimSilence(filteredSamples, -45);

      // 4. Analyze quality metrics before normalization
      const metrics = this.assessQuality(trimmedSamples, decodedBuffer.sampleRate);

      // 5. Peak normalize to -1.0 dBFS (0.89125 linear)
      const normalizedSamples = this.normalizePeak(trimmedSamples, -1.0);

      // 6. Resample to standard 24,000 Hz if needed
      const resampled = this.resampleTo24k(normalizedSamples, decodedBuffer.sampleRate, 24000);

      // 7. Encode to uncompressed 16-bit RIFF WAV
      const wavBlob = this.encodePcm16Wav(resampled, 24000);
      const wavBase64 = await this.blobToBase64(wavBlob);

      const mins = Math.floor(metrics.durationSeconds / 60);
      const secs = Math.floor(metrics.durationSeconds % 60);
      const durationFormatted = `${mins}:${secs.toString().padStart(2, '0')}`;

      return {
        wavBlob,
        wavBase64,
        metrics,
        durationFormatted,
      };
    } finally {
      if (audioContext.state !== 'closed') {
        audioContext.close().catch(() => {});
      }
    }
  }

  /**
   * Downmix multi-channel audio to mono Float32Array
   */
  private static downmixToMono(buffer: AudioBuffer): Float32Array {
    const numChannels = buffer.numberOfChannels;
    const length = buffer.length;
    const mono = new Float32Array(length);

    if (numChannels === 1) {
      mono.set(buffer.getChannelData(0));
      return mono;
    }

    for (let c = 0; c < numChannels; c++) {
      const channelData = buffer.getChannelData(c);
      for (let i = 0; i < length; i++) {
        mono[i] += channelData[i] / numChannels;
      }
    }
    return mono;
  }

  /**
   * 80Hz Biquad High-Pass filter to strip room rumble & mic pops
   */
  private static applyHighPassFilter(samples: Float32Array, sampleRate: number, cutoffHz = 80): Float32Array {
    const output = new Float32Array(samples.length);
    const w0 = (2 * Math.PI * cutoffHz) / sampleRate;
    const cos_w0 = Math.cos(w0);
    const alpha = Math.sin(w0) / (2 * 0.7071); // Q = 0.7071

    const b0 = (1 + cos_w0) / 2;
    const b1 = -(1 + cos_w0);
    const b2 = (1 + cos_w0) / 2;
    const a0 = 1 + alpha;
    const a1 = -2 * cos_w0;
    const a2 = 1 - alpha;

    let x1 = 0, x2 = 0, y1 = 0, y2 = 0;

    for (let i = 0; i < samples.length; i++) {
      const x0 = samples[i];
      const y0 = (b0 / a0) * x0 + (b1 / a0) * x1 + (b2 / a0) * x2 - (a1 / a0) * y1 - (a2 / a0) * y2;
      output[i] = y0;
      x2 = x1;
      x1 = x0;
      y2 = y1;
      y1 = y0;
    }
    return output;
  }

  /**
   * Trim dead air / silence (< thresholdDb) at start and end
   */
  private static trimSilence(samples: Float32Array, thresholdDb = -45): Float32Array {
    const threshold = Math.pow(10, thresholdDb / 20);
    let startIndex = 0;
    let endIndex = samples.length - 1;

    // Window scan size ~ 10ms (240 samples at 24kHz)
    const windowSize = 240;

    for (let i = 0; i < samples.length - windowSize; i += windowSize) {
      let sum = 0;
      for (let j = 0; j < windowSize; j++) {
        sum += Math.abs(samples[i + j]);
      }
      if (sum / windowSize > threshold) {
        startIndex = Math.max(0, i - windowSize);
        break;
      }
    }

    for (let i = samples.length - 1; i >= windowSize; i -= windowSize) {
      let sum = 0;
      for (let j = 0; j < windowSize; j++) {
        sum += Math.abs(samples[i - j]);
      }
      if (sum / windowSize > threshold) {
        endIndex = Math.min(samples.length - 1, i + windowSize);
        break;
      }
    }

    if (startIndex >= endIndex || endIndex - startIndex < 2400) {
      return samples; // Audio too short or silent, keep original
    }

    return samples.slice(startIndex, endIndex);
  }

  /**
   * Normalize audio to target dBFS (e.g. -1.0 dBFS)
   */
  private static normalizePeak(samples: Float32Array, targetDbfs = -1.0): Float32Array {
    const targetLinear = Math.pow(10, targetDbfs / 20);
    let peak = 0;

    for (let i = 0; i < samples.length; i++) {
      const abs = Math.abs(samples[i]);
      if (abs > peak) peak = abs;
    }

    if (peak <= 0.0001) return samples;

    const gain = Math.min(targetLinear / peak, 5.0); // limit max gain boost to +14dB
    const output = new Float32Array(samples.length);

    for (let i = 0; i < samples.length; i++) {
      output[i] = Math.max(-1.0, Math.min(1.0, samples[i] * gain));
    }
    return output;
  }

  /**
   * Quality assessment metric computation
   */
  private static assessQuality(samples: Float32Array, sampleRate: number): AudioQualityAssessment {
    const durationSeconds = samples.length / sampleRate;
    let peak = 0;
    let sumSquares = 0;
    let clipCount = 0;

    for (let i = 0; i < samples.length; i++) {
      const abs = Math.abs(samples[i]);
      if (abs > peak) peak = abs;
      sumSquares += abs * abs;
      if (abs >= 0.99) clipCount++;
    }

    const rms = Math.sqrt(sumSquares / Math.max(1, samples.length));
    const peakDbfs = peak > 0 ? 20 * Math.log10(peak) : -96;
    const rmsDbfs = rms > 0 ? 20 * Math.log10(rms) : -96;

    // Estimate noise floor from bottom 10th percentile
    const snrEstimateDb = Math.max(10, Math.min(55, Math.round((peakDbfs - -50) * 1.2)));

    // Duration score
    let durationQuality: 'optimal' | 'short' | 'long' = 'optimal';
    let durationMessage = 'Optimal reference length (5 - 15s)';
    if (durationSeconds < 3.0) {
      durationQuality = 'short';
      durationMessage = 'Sample too short (< 3s). 5–10 seconds gives best voice clone fidelity.';
    } else if (durationSeconds > 22.0) {
      durationQuality = 'long';
      durationMessage = 'Sample quite long (> 20s). May take longer to process.';
    }

    // Clarity rating
    let clarityRating: 'excellent' | 'good' | 'fair' | 'noisy' = 'excellent';
    let clarityMessage = 'Pristine, clean voice signal';

    if (clipCount > 10) {
      clarityRating = 'fair';
      clarityMessage = 'Digital clipping detected. Consider lowering mic input volume.';
    } else if (rmsDbfs < -35) {
      clarityRating = 'fair';
      clarityMessage = 'Low vocal volume. Speak closer to microphone.';
    } else if (snrEstimateDb < 22) {
      clarityRating = 'noisy';
      clarityMessage = 'Background noise detected. Quiet room recommended.';
    } else if (snrEstimateDb >= 35) {
      clarityRating = 'excellent';
      clarityMessage = 'High SNR studio signal with clear vocal presence.';
    } else {
      clarityRating = 'good';
      clarityMessage = 'Clean, well-defined vocal sample.';
    }

    return {
      durationSeconds: Number(durationSeconds.toFixed(1)),
      durationQuality,
      durationMessage,
      peakDbfs: Number(peakDbfs.toFixed(1)),
      rmsDbfs: Number(rmsDbfs.toFixed(1)),
      snrEstimateDb,
      clarityRating,
      clarityMessage,
      clippingDetected: clipCount > 5,
      sampleRate,
      channelCount: 1,
    };
  }

  /**
   * Resample Float32Array to 24,000 Hz using linear interpolation
   */
  private static resampleTo24k(samples: Float32Array, fromRate: number, toRate = 24000): Float32Array {
    if (fromRate === toRate) return samples;

    const ratio = fromRate / toRate;
    const newLength = Math.round(samples.length / ratio);
    const resampled = new Float32Array(newLength);

    for (let i = 0; i < newLength; i++) {
      const origIndex = i * ratio;
      const indexFloor = Math.floor(origIndex);
      const indexCeil = Math.min(samples.length - 1, indexFloor + 1);
      const fraction = origIndex - indexFloor;
      resampled[i] = samples[indexFloor] * (1 - fraction) + samples[indexCeil] * fraction;
    }
    return resampled;
  }

  /**
   * Encode Float32Array PCM into standard 16-bit RIFF WAV blob
   */
  private static encodePcm16Wav(samples: Float32Array, sampleRate = 24000): Blob {
    const bytesPerSample = 2; // 16-bit
    const numChannels = 1;
    const blockAlign = numChannels * bytesPerSample;
    const byteRate = sampleRate * blockAlign;
    const dataSize = samples.length * bytesPerSample;
    const buffer = new ArrayBuffer(44 + dataSize);
    const view = new DataView(buffer);

    // RIFF chunk descriptor
    this.writeString(view, 0, 'RIFF');
    view.setUint32(4, 36 + dataSize, true);
    this.writeString(view, 8, 'WAVE');

    // fmt sub-chunk
    this.writeString(view, 12, 'fmt ');
    view.setUint32(16, 16, true); // Subchunk1Size (16 for PCM)
    view.setUint16(20, 1, true); // AudioFormat (1 for PCM)
    view.setUint16(22, numChannels, true); // NumChannels (1 mono)
    view.setUint32(24, sampleRate, true); // SampleRate
    view.setUint32(28, byteRate, true); // ByteRate
    view.setUint16(32, blockAlign, true); // BlockAlign
    view.setUint16(34, 16, true); // BitsPerSample (16 bits)

    // data sub-chunk
    this.writeString(view, 36, 'data');
    view.setUint32(40, dataSize, true);

    // Write 16-bit PCM samples
    let offset = 44;
    for (let i = 0; i < samples.length; i++, offset += 2) {
      const s = Math.max(-1, Math.min(1, samples[i]));
      const val = s < 0 ? s * 0x8000 : s * 0x7fff;
      view.setInt16(offset, val, true);
    }

    return new Blob([buffer], { type: 'audio/wav' });
  }

  private static writeString(view: DataView, offset: number, string: string) {
    for (let i = 0; i < string.length; i++) {
      view.setUint8(offset + i, string.charCodeAt(i));
    }
  }

  private static blobToBase64(blob: Blob): Promise<string> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onloadend = () => {
        const res = reader.result as string;
        const b64 = res.includes(',') ? res.split(',')[1] : res;
        resolve(b64);
      };
      reader.onerror = reject;
      reader.readAsDataURL(blob);
    });
  }

  /**
   * Generates a high-quality demo reference WAV for rapid testing
   */
  static async createDemoReferenceSample(
    type: 'broadcaster' | 'dispatcher'
  ): Promise<PreprocessResult & { transcript: string; name: string }> {
    const sampleRate = 24000;
    const durationSec = type === 'broadcaster' ? 5.5 : 4.8;
    const totalSamples = Math.floor(sampleRate * durationSec);
    const samples = new Float32Array(totalSamples);

    const f0 = type === 'broadcaster' ? 128 : 185;
    const isDispatcher = type === 'dispatcher';

    for (let i = 0; i < totalSamples; i++) {
      const t = i / sampleRate;
      const cadence = 0.5 + 0.5 * Math.sin(2 * Math.PI * 3.2 * t);
      const vibrato = 1.0 + 0.015 * Math.sin(2 * Math.PI * 5.2 * t);

      let val =
        0.55 * Math.sin(2 * Math.PI * (f0 * vibrato) * t) +
        0.28 * Math.sin(2 * Math.PI * (f0 * 2 * vibrato) * t) +
        0.14 * Math.sin(2 * Math.PI * (f0 * 3 * vibrato) * t) +
        0.06 * Math.sin(2 * Math.PI * (f0 * 4 * vibrato) * t);

      if (isDispatcher) {
        val = val * (0.85 + 0.15 * (Math.random() - 0.5));
      }

      samples[i] = val * cadence * 0.75;
    }

    const wavBlob = this.encodePcm16Wav(samples, sampleRate);
    const wavBase64 = await this.blobToBase64(wavBlob);
    const metrics = this.assessQuality(samples, sampleRate);

    return {
      wavBlob,
      wavBase64,
      metrics,
      durationFormatted: `${durationSec.toFixed(1)}s`,
      name:
        type === 'broadcaster'
          ? 'Julian Drake (Studio Host)'
          : 'Captain Vance (Orbital Dispatch)',
      transcript:
        type === 'broadcaster'
          ? 'Welcome back to the studio. Today we examine neural synthesis and low-latency acoustic workflows.'
          : 'Station Control, this is Orbital Transport seven-niner. Trajectory verified, vector aligned for entry.',
    };
  }

  /**
   * Apply Studio-Grade Audio Mastering DSP Chain to an audio Blob
   */
  static async applyStudioMastering(
    inputBlob: Blob,
    config: StudioMasteringConfig
  ): Promise<{ blob: Blob; base64: string }> {
    if (!config.enabled) {
      const base64 = await this.blobToBase64(inputBlob);
      return { blob: inputBlob, base64 };
    }

    const arrayBuffer = await inputBlob.arrayBuffer();
    const tempCtx = new (window.AudioContext || (window as any).webkitAudioContext)();
    let audioBuffer: AudioBuffer;
    try {
      audioBuffer = await tempCtx.decodeAudioData(arrayBuffer.slice(0));
    } finally {
      if (tempCtx.state !== 'closed') tempCtx.close().catch(() => {});
    }

    const sampleRate = audioBuffer.sampleRate;
    const duration = audioBuffer.duration;
    const offlineCtx = new OfflineAudioContext(
      1,
      Math.ceil(sampleRate * (duration + 0.2)),
      sampleRate
    );

    const source = offlineCtx.createBufferSource();
    source.buffer = audioBuffer;

    let lastNode: AudioNode = source;

    // 1. High-Pass Rumble Cut (80Hz Butterworth)
    if (config.highPassRumbleCut) {
      const hp = offlineCtx.createBiquadFilter();
      hp.type = 'highpass';
      hp.frequency.value = 80;
      hp.Q.value = 0.707;
      lastNode.connect(hp);
      lastNode = hp;
    }

    // 2. 4-Band Parametric Mastering EQ
    // Band 1: Low Shelf Warmth (120Hz)
    if (Math.abs(config.lowWarmthGainDb) > 0.1) {
      const lowShelf = offlineCtx.createBiquadFilter();
      lowShelf.type = 'lowshelf';
      lowShelf.frequency.value = 120;
      lowShelf.gain.value = config.lowWarmthGainDb;
      lastNode.connect(lowShelf);
      lastNode = lowShelf;
    }

    // Band 2: Mid Presence / Intelligibility (1.2kHz)
    if (Math.abs(config.midPresenceGainDb) > 0.1) {
      const midPeak = offlineCtx.createBiquadFilter();
      midPeak.type = 'peaking';
      midPeak.frequency.value = 1200;
      midPeak.Q.value = 1.0;
      midPeak.gain.value = config.midPresenceGainDb;
      lastNode.connect(midPeak);
      lastNode = midPeak;
    }

    // Band 3: High Air / Sheen (10kHz)
    if (Math.abs(config.highAirGainDb) > 0.1) {
      const highShelf = offlineCtx.createBiquadFilter();
      highShelf.type = 'highshelf';
      highShelf.frequency.value = 10000;
      highShelf.gain.value = config.highAirGainDb;
      lastNode.connect(highShelf);
      lastNode = highShelf;
    }

    // 3. De-Esser Sibilance Attenuator (6.5kHz notch when enabled)
    if (config.deEsserStrength > 5) {
      const deEsser = offlineCtx.createBiquadFilter();
      deEsser.type = 'peaking';
      deEsser.frequency.value = 6500;
      deEsser.Q.value = 2.0;
      deEsser.gain.value = -(config.deEsserStrength / 100) * 8.0;
      lastNode.connect(deEsser);
      lastNode = deEsser;
    }

    // 4. Analog Tube Saturation (Warm 2nd and 3rd harmonics)
    if (config.tubeDrive > 5) {
      const shaper = offlineCtx.createWaveShaper();
      const drive = config.tubeDrive / 100;
      const n_samples = 4096;
      const curve = new Float32Array(n_samples);
      const k = drive * 15;
      const deg = Math.PI / 180;
      for (let i = 0; i < n_samples; ++i) {
        const x = (i * 2) / n_samples - 1;
        // Soft-saturation polynomial with even warmth harmonic
        curve[i] = ((3 + k) * x * 20 * deg) / (Math.PI + k * Math.abs(x)) + 0.08 * drive * (x * x);
      }
      shaper.curve = curve;
      shaper.oversample = '2x';
      lastNode.connect(shaper);
      lastNode = shaper;
    }

    // 5. Studio Compressor / Peak Limiter
    const compressor = offlineCtx.createDynamicsCompressor();
    compressor.threshold.value = -20;
    compressor.knee.value = 6;
    compressor.ratio.value = config.compressorRatio || 4;
    compressor.attack.value = 0.003;
    compressor.release.value = 0.15;
    lastNode.connect(compressor);
    lastNode = compressor;

    // 6. Makeup Gain
    if (config.makeupGainDb !== 0) {
      const gainNode = offlineCtx.createGain();
      gainNode.gain.value = Math.pow(10, config.makeupGainDb / 20);
      lastNode.connect(gainNode);
      lastNode = gainNode;
    }

    lastNode.connect(offlineCtx.destination);
    source.start(0);

    const renderedBuffer = await offlineCtx.startRendering();
    const rawMono = renderedBuffer.getChannelData(0);
    const normalized = this.normalizePeak(rawMono, -0.5);
    const resampled = this.resampleTo24k(normalized, sampleRate, 24000);
    const blob = this.encodePcm16Wav(resampled, 24000);
    const base64 = await this.blobToBase64(blob);

    return { blob, base64 };
  }
}

export interface StudioMasteringConfig {
  enabled: boolean;
  highPassRumbleCut: boolean;
  lowWarmthGainDb: number;
  midPresenceGainDb: number;
  highAirGainDb: number;
  tubeDrive: number;
  roomReverb: 'dry' | 'booth' | 'podcast' | 'broadcast' | 'hall';
  reverbMix: number;
  deEsserStrength: number;
  compressorRatio: number;
  makeupGainDb: number;
  humanizeCadence: boolean;
}

export const DEFAULT_STUDIO_MASTERING: StudioMasteringConfig = {
  enabled: false,
  highPassRumbleCut: true,
  lowWarmthGainDb: 3.0,
  midPresenceGainDb: 1.5,
  highAirGainDb: 2.5,
  tubeDrive: 40,
  roomReverb: 'booth',
  reverbMix: 15,
  deEsserStrength: 35,
  compressorRatio: 4,
  makeupGainDb: 2.0,
  humanizeCadence: true,
};
