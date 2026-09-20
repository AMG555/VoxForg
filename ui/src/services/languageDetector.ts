/**
 * VoxForg Language Detector
 * Real-time script & dialect detection with support for:
 * - Native Indic scripts (Malayalam, Hindi, Tamil, Telugu, Kannada, Bengali, Marathi, Gujarati, Urdu, Punjabi)
 * - Romanized Indic Dialects (Manglish, Hinglish, Tanglish, Kanglish, Tenglish)
 * - World languages (English, Spanish, French, German, Japanese, Chinese)
 */

export interface DetectedLanguage {
  code: string;
  name: string;
  flag: string;
  isRomanizedDialect: boolean;
  nativeTargetCode?: string;
  suggestedVoiceId?: string;
  confidence: number;
  sampleKeywordsFound?: string[];
}

export interface LanguageOption {
  code: string;
  name: string;
  flag: string;
  nativeScript?: string;
  isRomanized?: boolean;
}

export const SUPPORTED_LANGUAGES: LanguageOption[] = [
  { code: 'AUTO', name: 'Auto-Detect Language', flag: '🌐' },
  { code: 'ml-Latn', name: 'Manglish (Malayalam Romanized)', flag: '🌴', isRomanized: true },
  { code: 'ml-IN', name: 'Malayalam (മലയാളം)', flag: '🇮🇳', nativeScript: 'മലയാളം' },
  { code: 'hi-Latn', name: 'Hinglish (Hindi Romanized)', flag: '🇮🇳', isRomanized: true },
  { code: 'hi-IN', name: 'Hindi (हिन्दी)', flag: '🇮🇳', nativeScript: 'हिन्दी' },
  { code: 'ta-Latn', name: 'Tanglish (Tamil Romanized)', flag: '🇮🇳', isRomanized: true },
  { code: 'ta-IN', name: 'Tamil (தமிழ்)', flag: '🇮🇳', nativeScript: 'தமிழ்' },
  { code: 'te-IN', name: 'Telugu (తెలుగు)', flag: '🇮🇳', nativeScript: 'తెలుగు' },
  { code: 'kn-IN', name: 'Kannada (ಕನ್ನಡ)', flag: '🇮🇳', nativeScript: 'ಕನ್ನಡ' },
  { code: 'bn-IN', name: 'Bengali (বাংলা)', flag: '🇮🇳', nativeScript: 'বাংলা' },
  { code: 'mr-IN', name: 'Marathi (मराठी)', flag: '🇮🇳', nativeScript: 'मराठी' },
  { code: 'gu-IN', name: 'Gujarati (ગુજરાતી)', flag: '🇮🇳', nativeScript: 'ગુજરાતી' },
  { code: 'ur-IN', name: 'Urdu (اردو)', flag: '🇮🇳', nativeScript: 'اردو' },
  { code: 'pa-IN', name: 'Punjabi (ਪੰਜਾਬੀ)', flag: '🇮🇳', nativeScript: 'ਪੰਜਾਬੀ' },
  { code: 'en-US', name: 'English (US)', flag: '🇺🇸' },
  { code: 'en-GB', name: 'English (UK)', flag: '🇬🇧' },
  { code: 'en-IN', name: 'Indian English', flag: '🇮🇳' },
  { code: 'es-ES', name: 'Spanish (Español)', flag: '🇪🇸' },
  { code: 'fr-FR', name: 'French (Français)', flag: '🇫🇷' },
  { code: 'de-DE', name: 'German (Deutsch)', flag: '🇩🇪' },
  { code: 'ja-JP', name: 'Japanese (日本語)', flag: '🇯🇵' },
  { code: 'zh-CN', name: 'Mandarin Chinese (中文)', flag: '🇨🇳' },
];

// Lexicon for Romanized Manglish (Malayalam written in Latin script)
const MANGLISH_KEYWORDS = new Set([
  'adipoli', 'adpoli', 'kidilam', 'machane', 'chunke', 'pwoli', 'scene', 'sceneanu', 'sceneaanu',
  'enthokke', 'vishesham', 'njan', 'njangal', 'ninte', 'ningal', 'ivide', 'avide', 'evide', 'evideya',
  'poda', 'mone', 'chetta', 'chechi', 'aano', 'und', 'undu', 'illa', 'raksha', 'poli', 'valare',
  'nannayi', 'manassilayi', 'engane', 'ippo', 'ippol', 'kollam', 'thanne', 'sathyam', 'pinne',
  'nokku', 'parayu', 'parayam', 'enthaanu', 'entha', 'kidu', 'mass', 'chumma', 'parayamo',
  'ariyilla', 'sheriyaanu', 'sheriyaa', 'veruthe', 'thante', 'cheyyanam', 'enthoru', 'nalla',
  'kollaam', 'thannathaane', 'kaaryam', 'sukhamano', 'namaskaram', 'oru', 'ithu', 'athu', 'nammude',
  'naale', 'innu', 'innale', 'varumo', 'poyalo', 'kelkkamo', 'setaayi', 'set aayi', 'aliya', 'aliyan',
  'dhaivame', 'ente', 'ponno', 'enteponno', 'kollalo', 'polichu'
]);

// Lexicon for Romanized Hinglish (Hindi written in Latin script)
const HINGLISH_KEYWORDS = new Set([
  'bhai', 'kya', 'haal', 'chal', 'ekdum', 'mast', 'achha', 'accha', 'theek', 'shukriya',
  'dhanyawad', 'yaar', 'kaise', 'kahan', 'karo', 'suno', 'dekho', 'kuch', 'nahi', 'nahin',
  'hota', 'samajh', 'gaya', 'chalega', 'bahut', 'badhiya', 'jhakaas', 'arre', 'arey', 'bolo',
  'apna', 'apne', 'kaam', 'karna', 'kardo', 'zara', 'sunao', 'sabko', 'hoga', 'namaste',
  'dost', 'chalo', 'batao', 'sahi', 'hai', 'hain', 'mujhe', 'tumhe', 'hum', 'aap', 'mera',
  'tera', 'bolo', 'sun', 'karega', 'lao', 'khabar', 'paani', 'arrewaah', 'waah'
]);

// Lexicon for Romanized Tanglish (Tamil written in Latin script)
const TANGLISH_KEYWORDS = new Set([
  'machi', 'vanakkam', 'epdi', 'irukka', 'semma', 'vera', 'level', 'veralevel', 'nandri',
  'theriyum', 'solla', 'vaanga', 'ponga', 'thalaiva', 'supera', 'romba', 'illai', 'podu',
  'paravala', 'solunga', 'kudunga', 'nanba', 'nanbi', 'ennada', 'enna', 'inga', 'anga',
  'seri', 'amma', 'appa', 'anna', 'machan', 'kandippa', 'apdiya', 'kelunga'
]);

// Lexicon for European common words
const SPANISH_KEYWORDS = new Set([
  'hola', 'buenos', 'dias', 'tardes', 'noches', 'gracias', 'por', 'favor', 'amigo', 'como',
  'estas', 'bien', 'donde', 'esta', 'adios', 'hasta', 'luego', 'senor', 'senora'
]);

const FRENCH_KEYWORDS = new Set([
  'bonjour', 'salut', 'merci', 'beaucoup', 'comment', 'allez', 'vous', 'bien', 'oui',
  'non', 's\'il', 'vous', 'plait', 'au', 'revoir', 'monsieur', 'madame'
]);

const GERMAN_KEYWORDS = new Set([
  'hallo', 'guten', 'tag', 'morgen', 'abend', 'danke', 'bitte', 'wie', 'geht\'s', 'tschuss',
  'auf', 'wiedersehen', 'ja', 'nein', 'freund', 'schon', 'wunderbar'
]);

export function detectLanguage(text: string): DetectedLanguage {
  const trimmed = text.trim();
  if (!trimmed) {
    return {
      code: 'en-US',
      name: 'English (US)',
      flag: '🇺🇸',
      isRomanizedDialect: false,
      suggestedVoiceId: 'en-US-AriaNeural',
      confidence: 1.0,
    };
  }

  // 1. Check Native Script Unicode Ranges
  let malayalamChars = 0;
  let devanagariChars = 0;
  let tamilChars = 0;
  let teluguChars = 0;
  let kannadaChars = 0;
  let bengaliChars = 0;
  let gujaratiChars = 0;
  let gurmukhiChars = 0;
  let arabicChars = 0;
  let cjkChars = 0;
  let japaneseChars = 0;
  let totalScriptChars = 0;

  for (let i = 0; i < trimmed.length; i++) {
    const code = trimmed.charCodeAt(i);
    // Malayalam: U+0D00 - U+0D7F
    if (code >= 0x0d00 && code <= 0x0d7f) {
      malayalamChars++;
      totalScriptChars++;
    }
    // Devanagari (Hindi, Marathi): U+0900 - U+097F
    else if (code >= 0x0900 && code <= 0x097f) {
      devanagariChars++;
      totalScriptChars++;
    }
    // Tamil: U+0B80 - U+0BFF
    else if (code >= 0x0b80 && code <= 0x0bff) {
      tamilChars++;
      totalScriptChars++;
    }
    // Telugu: U+0C00 - U+0C7F
    else if (code >= 0x0c00 && code <= 0x0c7f) {
      teluguChars++;
      totalScriptChars++;
    }
    // Kannada: U+0C80 - U+0CFF
    else if (code >= 0x0c80 && code <= 0x0cff) {
      kannadaChars++;
      totalScriptChars++;
    }
    // Bengali: U+0980 - U+09FF
    else if (code >= 0x0980 && code <= 0x09ff) {
      bengaliChars++;
      totalScriptChars++;
    }
    // Gujarati: U+0A80 - U+0AFF
    else if (code >= 0x0a80 && code <= 0x0aff) {
      gujaratiChars++;
      totalScriptChars++;
    }
    // Gurmukhi (Punjabi): U+0A00 - U+0A7F
    else if (code >= 0x0a00 && code <= 0x0a7f) {
      gurmukhiChars++;
      totalScriptChars++;
    }
    // Arabic / Urdu: U+0600 - U+06FF
    else if (code >= 0x0600 && code <= 0x06ff) {
      arabicChars++;
      totalScriptChars++;
    }
    // Japanese Hiragana & Katakana: U+3040 - U+30FF
    else if ((code >= 0x3040 && code <= 0x309f) || (code >= 0x30a0 && code <= 0x30ff)) {
      japaneseChars++;
      totalScriptChars++;
    }
    // CJK Unified Ideographs: U+4E00 - U+9FFF
    else if (code >= 0x4e00 && code <= 0x9fff) {
      cjkChars++;
      totalScriptChars++;
    }
  }

  // Native Script Matches (dominant count)
  if (malayalamChars >= 2 || (malayalamChars > 0 && totalScriptChars === malayalamChars)) {
    return {
      code: 'ml-IN',
      name: 'Malayalam (മലയാളം)',
      flag: '🇮🇳',
      isRomanizedDialect: false,
      suggestedVoiceId: 'ml-IN-SobhanaNeural',
      confidence: Math.min(1.0, 0.7 + (malayalamChars / trimmed.length) * 0.3),
    };
  }

  if (tamilChars >= 2 || (tamilChars > 0 && totalScriptChars === tamilChars)) {
    return {
      code: 'ta-IN',
      name: 'Tamil (தமிழ்)',
      flag: '🇮🇳',
      isRomanizedDialect: false,
      suggestedVoiceId: 'ta-IN-PallaviNeural',
      confidence: Math.min(1.0, 0.7 + (tamilChars / trimmed.length) * 0.3),
    };
  }

  if (teluguChars >= 2 || (teluguChars > 0 && totalScriptChars === teluguChars)) {
    return {
      code: 'te-IN',
      name: 'Telugu (తెలుగు)',
      flag: '🇮🇳',
      isRomanizedDialect: false,
      suggestedVoiceId: 'te-IN-ShrutiNeural',
      confidence: Math.min(1.0, 0.7 + (teluguChars / trimmed.length) * 0.3),
    };
  }

  if (kannadaChars >= 2 || (kannadaChars > 0 && totalScriptChars === kannadaChars)) {
    return {
      code: 'kn-IN',
      name: 'Kannada (ಕನ್ನಡ)',
      flag: '🇮🇳',
      isRomanizedDialect: false,
      suggestedVoiceId: 'kn-IN-SapnaNeural',
      confidence: Math.min(1.0, 0.7 + (kannadaChars / trimmed.length) * 0.3),
    };
  }

  if (devanagariChars >= 2 || (devanagariChars > 0 && totalScriptChars === devanagariChars)) {
    return {
      code: 'hi-IN',
      name: 'Hindi (हिन्दी)',
      flag: '🇮🇳',
      isRomanizedDialect: false,
      suggestedVoiceId: 'hi-IN-SwaraNeural',
      confidence: Math.min(1.0, 0.7 + (devanagariChars / trimmed.length) * 0.3),
    };
  }

  if (bengaliChars >= 2 || (bengaliChars > 0 && totalScriptChars === bengaliChars)) {
    return {
      code: 'bn-IN',
      name: 'Bengali (বাংলা)',
      flag: '🇮🇳',
      isRomanizedDialect: false,
      suggestedVoiceId: 'bn-IN-TanishaaNeural',
      confidence: Math.min(1.0, 0.7 + (bengaliChars / trimmed.length) * 0.3),
    };
  }

  if (gujaratiChars >= 2 || (gujaratiChars > 0 && totalScriptChars === gujaratiChars)) {
    return {
      code: 'gu-IN',
      name: 'Gujarati (ગુજરાતી)',
      flag: '🇮🇳',
      isRomanizedDialect: false,
      suggestedVoiceId: 'gu-IN-DhwaniNeural',
      confidence: Math.min(1.0, 0.7 + (gujaratiChars / trimmed.length) * 0.3),
    };
  }

  if (gurmukhiChars >= 2 || (gurmukhiChars > 0 && totalScriptChars === gurmukhiChars)) {
    return {
      code: 'pa-IN',
      name: 'Punjabi (ਪੰਜਾਬੀ)',
      flag: '🇮🇳',
      isRomanizedDialect: false,
      suggestedVoiceId: 'pa-IN-VaaniNeural',
      confidence: Math.min(1.0, 0.7 + (gurmukhiChars / trimmed.length) * 0.3),
    };
  }

  if (arabicChars >= 2 || (arabicChars > 0 && totalScriptChars === arabicChars)) {
    return {
      code: 'ur-IN',
      name: 'Urdu (اردو)',
      flag: '🇮🇳',
      isRomanizedDialect: false,
      suggestedVoiceId: 'ur-IN-GulNeural',
      confidence: Math.min(1.0, 0.7 + (arabicChars / trimmed.length) * 0.3),
    };
  }

  if (japaneseChars >= 1) {
    return {
      code: 'ja-JP',
      name: 'Japanese (日本語)',
      flag: '🇯🇵',
      isRomanizedDialect: false,
      suggestedVoiceId: 'ja-JP-NanamiNeural',
      confidence: 0.95,
    };
  }

  if (cjkChars >= 2) {
    return {
      code: 'zh-CN',
      name: 'Mandarin Chinese (中文)',
      flag: '🇨🇳',
      isRomanizedDialect: false,
      suggestedVoiceId: 'zh-CN-XiaoxiaoNeural',
      confidence: 0.95,
    };
  }

  // 2. Latin Script: Check for Romanized Indic Dialects (Manglish, Hinglish, Tanglish)
  const words = trimmed
    .toLowerCase()
    .replace(/[^\w\s']/g, ' ')
    .split(/\s+/)
    .filter((w) => w.length > 1);

  const matchedManglish: string[] = [];
  const matchedHinglish: string[] = [];
  const matchedTanglish: string[] = [];
  const matchedSpanish: string[] = [];
  const matchedFrench: string[] = [];
  const matchedGerman: string[] = [];

  for (const word of words) {
    if (MANGLISH_KEYWORDS.has(word)) matchedManglish.push(word);
    if (HINGLISH_KEYWORDS.has(word)) matchedHinglish.push(word);
    if (TANGLISH_KEYWORDS.has(word)) matchedTanglish.push(word);
    if (SPANISH_KEYWORDS.has(word)) matchedSpanish.push(word);
    if (FRENCH_KEYWORDS.has(word)) matchedFrench.push(word);
    if (GERMAN_KEYWORDS.has(word)) matchedGerman.push(word);
  }

  // Manglish Detection: If 1+ distinct Manglish keywords match and exceeds other indic romanizations
  if (matchedManglish.length > 0 && matchedManglish.length >= matchedHinglish.length && matchedManglish.length >= matchedTanglish.length) {
    const confidence = Math.min(0.98, 0.65 + matchedManglish.length * 0.12);
    return {
      code: 'ml-Latn',
      name: 'Manglish (Malayalam Romanized)',
      flag: '🌴',
      isRomanizedDialect: true,
      nativeTargetCode: 'ml-IN',
      suggestedVoiceId: 'ml-IN-SobhanaNeural',
      confidence,
      sampleKeywordsFound: matchedManglish,
    };
  }

  // Hinglish Detection
  if (matchedHinglish.length > 0 && matchedHinglish.length >= matchedTanglish.length) {
    const confidence = Math.min(0.98, 0.65 + matchedHinglish.length * 0.12);
    return {
      code: 'hi-Latn',
      name: 'Hinglish (Hindi Romanized)',
      flag: '🇮🇳',
      isRomanizedDialect: true,
      nativeTargetCode: 'hi-IN',
      suggestedVoiceId: 'hi-IN-SwaraNeural',
      confidence,
      sampleKeywordsFound: matchedHinglish,
    };
  }

  // Tanglish Detection
  if (matchedTanglish.length > 0) {
    const confidence = Math.min(0.98, 0.65 + matchedTanglish.length * 0.12);
    return {
      code: 'ta-Latn',
      name: 'Tanglish (Tamil Romanized)',
      flag: '🇮🇳',
      isRomanizedDialect: true,
      nativeTargetCode: 'ta-IN',
      suggestedVoiceId: 'ta-IN-PallaviNeural',
      confidence,
      sampleKeywordsFound: matchedTanglish,
    };
  }

  // European languages check
  if (matchedSpanish.length >= 2) {
    return {
      code: 'es-ES',
      name: 'Spanish (Español)',
      flag: '🇪🇸',
      isRomanizedDialect: false,
      suggestedVoiceId: 'es-ES-AlvaroNeural',
      confidence: 0.85,
    };
  }

  if (matchedFrench.length >= 2) {
    return {
      code: 'fr-FR',
      name: 'French (Français)',
      flag: '🇫🇷',
      isRomanizedDialect: false,
      suggestedVoiceId: 'fr-FR-DeniseNeural',
      confidence: 0.85,
    };
  }

  if (matchedGerman.length >= 2) {
    return {
      code: 'de-DE',
      name: 'German (Deutsch)',
      flag: '🇩🇪',
      isRomanizedDialect: false,
      suggestedVoiceId: 'de-DE-KatjaNeural',
      confidence: 0.85,
    };
  }

  // Default Latin: English
  return {
    code: 'en-US',
    name: 'English (US)',
    flag: '🇺🇸',
    isRomanizedDialect: false,
    suggestedVoiceId: 'en-US-AriaNeural',
    confidence: 0.9,
  };
}
