/**
 * Multilingual Slang, Idiom & Colloquial Speech Presets
 *
 * Provides authentic, natural conversational dialogues across global dialects
 * so synthetic voices speak with real cadence, regional expressions, and idioms
 * instead of robotic, stilted textbook sentences.
 */

export interface SlangPreset {
  id: string;
  language: string;
  langCode: string;
  dialect: string;
  flag: string;
  title: string;
  description: string;
  sampleText: string;
  recommendedVoice: string;
  speed: number;
  pitch: number;
  cadenceHints: string;
}

export const MULTILINGUAL_SLANG_PRESETS: SlangPreset[] = [
  // ─── English Dialects ────────────────────────────────────────────────────────
  {
    id: 'en-us-casual',
    language: 'English',
    langCode: 'en-US',
    dialect: 'US Casual / West Coast',
    flag: '🇺🇸',
    title: 'Everyday Casual Conversation',
    description: 'Natural American informal cadence with contractions and relaxed rhythm.',
    sampleText: "Honestly, it's kinda wild how clean this voice sounds. We're gonna wrap up the mix, grab some coffee, and test the podcast stream.",
    recommendedVoice: 'en-US-AriaNeural',
    speed: 1.02,
    pitch: 0.0,
    cadenceHints: 'Relaxed vowels, conversational pauses, gentle ending pitch drop',
  },
  {
    id: 'en-uk-colloquial',
    language: 'English',
    langCode: 'en-GB',
    dialect: 'British / London Street',
    flag: '🇬🇧',
    title: 'London Colloquial & Tag Questions',
    description: 'Authentic UK phrasing featuring tag questions and colloquial exclamations.',
    sampleText: "Blimey, that's proper brilliant, innit? Don't fret mate, we'll have a quick brew and sort the master levels in two ticks.",
    recommendedVoice: 'en-GB-SoniaNeural',
    speed: 1.04,
    pitch: 0.5,
    cadenceHints: 'Dynamic pitch inflection on tag questions, crisp consonants',
  },
  {
    id: 'en-au-slang',
    language: 'English',
    langCode: 'en-AU',
    dialect: 'Australian Strine',
    flag: '🇦🇺',
    title: 'Aussie Casual & Diminutives',
    description: 'Iconic Australian conversational cadence with authentic regional slang.',
    sampleText: "G'day mate! Grab some brekkie this arvo, no worries at all. We'll give the new neural pipeline a fair go.",
    recommendedVoice: 'en-AU-NatashaNeural',
    speed: 0.98,
    pitch: -0.5,
    cadenceHints: 'Rising inflection on statements (Australian high rising terminal), friendly drawl',
  },
  {
    id: 'en-in-hinglish',
    language: 'English (Hinglish)',
    langCode: 'en-IN',
    dialect: 'Mumbai / Delhi Hinglish',
    flag: '🇮🇳',
    title: 'Indian English & Hinglish Expressions',
    description: 'Vibrant metropolitan Hinglish dialogue with expressive emotional particles.',
    sampleText: "Arre yaar, this studio jugaad is totally bindaas! Chalo, let's finish the sound check fatafat before the meeting starts.",
    recommendedVoice: 'en-IN-NeerjaNeural',
    speed: 1.02,
    pitch: 0.0,
    cadenceHints: 'Syllable-timed rhythm, enthusiastic emotional emphasis on particles',
  },

  // ─── Spanish Dialects ────────────────────────────────────────────────────────
  {
    id: 'es-es-castilian',
    language: 'Spanish',
    langCode: 'es-ES',
    dialect: 'Castilian (Madrid / Central)',
    flag: '🇪🇸',
    title: 'Castilian Spanish Colloquial',
    description: 'Natural peninsular Spanish with authentic colloquial affirmations and fillers.',
    sampleText: "¡Hombre, qué guay ha quedado esto! Vale, me parece genial. Venga, nos tomamos un café y afinamos el master.",
    recommendedVoice: 'es-ES-ElviraNeural',
    speed: 1.05,
    pitch: 0.2,
    cadenceHints: 'Crisp distinctions, rapid sentence pace, energetic sentence-end lift',
  },
  {
    id: 'es-mx-chilango',
    language: 'Spanish',
    langCode: 'es-MX',
    dialect: 'Mexican (Mexico City / Urban)',
    flag: '🇲🇽',
    title: 'Mexican Street & Casual Slang',
    description: 'Warm, melodious Mexican speech with popular regional vocabulary and cadence.',
    sampleText: "¡Qué onda güey! Neta que este audio suena bien chido. Aguántame tantito y ahorita mismo lo dejamos al cien.",
    recommendedVoice: 'es-MX-DaliaNeural',
    speed: 1.0,
    pitch: 0.0,
    cadenceHints: 'Musical intonation contours, elongated penultimate syllables on emphasis',
  },
  {
    id: 'es-ar-rioplatense',
    language: 'Spanish',
    langCode: 'es-AR',
    dialect: 'Argentine (Buenos Aires Rioplatense)',
    flag: '🇦🇷',
    title: 'Rioplatense / Porteño Dialogue',
    description: 'Distinctive Argentine cadence with Italian-influenced intonation and voseo.',
    sampleText: "¡Che, qué hacés! Mirá, la verdad que este sintetizador es una masa. De una que lo usamos para el podcast.",
    recommendedVoice: 'es-AR-ElenaNeural',
    speed: 0.98,
    pitch: 0.4,
    cadenceHints: 'Strong melodic waves with Italian-influenced cadences and expressive vowel elongation',
  },

  // ─── French Dialects ─────────────────────────────────────────────────────────
  {
    id: 'fr-fr-parisian',
    language: 'French',
    langCode: 'fr-FR',
    dialect: 'Parisian / Metropolitan Colloquial',
    flag: '🇫🇷',
    title: 'French Urban Slang & Verlan',
    description: 'Modern Parisian speech with colloquial contractions and popular expressions.',
    sampleText: "Ouais grave, c'est du boulot de ouf ! Le rendu sonore est carrément nickel. T'sais quoi ? On valide la piste direct.",
    recommendedVoice: 'fr-FR-DeniseNeural',
    speed: 1.02,
    pitch: 0.0,
    cadenceHints: 'Liaisons naturally softened in casual register, rhythmic stress on final syllables',
  },
  {
    id: 'fr-ca-quebec',
    language: 'French',
    langCode: 'fr-CA',
    dialect: 'Quebec Joual / Montreal',
    flag: '🇨🇦',
    title: 'Quebecois Colloquial Expressions',
    description: 'Distinctive French Canadian speech patterns with authentic regional idioms.',
    sampleText: "Mon chum m'a dit que c't'affaire-là est pas pire pantoute ! Attache ta tuque, on part le studio tout de suite.",
    recommendedVoice: 'fr-CA-SylvieNeural',
    speed: 1.0,
    pitch: -0.3,
    cadenceHints: 'Open vowels, rounded timbre, lively conversational pace',
  },

  // ─── German Dialects ─────────────────────────────────────────────────────────
  {
    id: 'de-de-berlin',
    language: 'German',
    langCode: 'de-DE',
    dialect: 'German Everyday Casual',
    flag: '🇩🇪',
    title: 'German Colloquial Dialogue',
    description: 'Authentic German conversational style without stiff bureaucratic phrasing.',
    sampleText: "Na alter, das ist echt krass! Alles klar, kein Ding. Lass uns mal kurz reinhören und dann geht's direkt los.",
    recommendedVoice: 'de-DE-KatjaNeural',
    speed: 1.03,
    pitch: 0.0,
    cadenceHints: 'Conversational contractions (mach es -> mach\'s), punchy consonants',
  },

  // ─── Japanese Dialects ───────────────────────────────────────────────────────
  {
    id: 'ja-jp-casual',
    language: 'Japanese',
    langCode: 'ja-JP',
    dialect: 'Tokyo / Casual Youth Japanese',
    flag: '🇯🇵',
    title: 'Conversational Japanese & Fillers',
    description: 'Natural spoken Japanese with authentic particles, colloquial contractions, and warmth.',
    sampleText: "マジでやばい！めっちゃいい音出てるじゃん。今日もお疲れ様、さっそくこのトラックで進めちゃおう！",
    recommendedVoice: 'ja-JP-NanamiNeural',
    speed: 1.04,
    pitch: 0.2,
    cadenceHints: 'Mora-timed cadence, natural rising terminal on casual interrogatives',
  },

  // ─── Chinese (Mandarin) ─────────────────────────────────────────────────────
  {
    id: 'zh-cn-colloquial',
    language: 'Chinese',
    langCode: 'zh-CN',
    dialect: 'Mandarin Colloquial & Web Slang',
    flag: '🇨🇳',
    title: 'Modern Colloquial Mandarin',
    description: 'Dynamic contemporary Chinese with popular colloquial expressions and lively tone.',
    sampleText: "好家伙，这AI声音也太给力了吧！音质杠杠的，简直666，赶紧把这段录音导出来听听。",
    recommendedVoice: 'zh-CN-XiaoxiaoNeural',
    speed: 1.05,
    pitch: 0.0,
    cadenceHints: 'Tone sandhi naturally integrated, enthusiastic conversational flow',
  },

  // ─── Hindi Dialects ──────────────────────────────────────────────────────────
  {
    id: 'hi-in-tapori',
    language: 'Hindi',
    langCode: 'hi-IN',
    dialect: 'Colloquial Hindi / Bollywood Street',
    flag: '🇮🇳',
    title: 'Expressive Hindi Colloquial',
    description: 'Lively, expressive conversational Hindi with popular everyday phrases.',
    sampleText: "अरे भाई, क्या झकास आवाज़ बनाई है! एकदम मस्त लग रहा है, फटाफट प्ले करो और सुनो।",
    recommendedVoice: 'hi-IN-SwaraNeural',
    speed: 1.0,
    pitch: 0.0,
    cadenceHints: 'Expressive emotional inflections, warm retroflex consonants',
  },

  // ─── Malayalam Dialects ──────────────────────────────────────────────────────
  {
    id: 'ml-in-kerala',
    language: 'Malayalam',
    langCode: 'ml-IN',
    dialect: 'Kerala (Kochi / Trivandrum Colloquial)',
    flag: '🇮🇳',
    title: 'Kerala Youth Slang & Conversational Flow (മലയാളം)',
    description: 'Authentic Malayalam colloquial speech with natural particles, lively rhythm, and warm native tone.',
    sampleText: "എന്റെ പൊന്നോ, ഈ വോക്സ്ഫോർഗ് സൗണ്ട് ഔട്ട്പുട്ട് അടിപൊളിയാണ് മച്ചാനേ! വർക്ക്ഫ്ലോ കംപ്ലീറ്റ് സെറ്റ് ആയി, ഇനി ഒട്ടും വൈകിക്കാതെ ലൈവ് സ്ട്രീം സ്റ്റാർട്ട് ചെയ്യാം.",
    recommendedVoice: 'ml-IN-SobhanaNeural',
    speed: 1.0,
    pitch: 0.0,
    cadenceHints: 'Natural rhythmic pauses at clause boundaries, warm native Malayalam inflection',
  },

  // ─── Tamil Dialects ──────────────────────────────────────────────────────────
  {
    id: 'ta-in-chennai',
    language: 'Tamil',
    langCode: 'ta-IN',
    dialect: 'Chennai (Madras Bashai / Colloquial)',
    flag: '🇮🇳',
    title: 'Chennai Colloquial & Conversational Tamil (தமிழ்)',
    description: 'Vibrant Tamil conversational speech with authentic particles and energetic rhythm.',
    sampleText: "மச்சி, இந்த வாய்ஸ் அவுட்புட் சும்மா வேற லெவல்ல இருக்குடா! செம்ம கிளீனா கேக்குது, உடனே நம்ம பாட்காஸ்ட் ஸ்டார்ட் பண்ணிடலாம்.",
    recommendedVoice: 'ta-IN-PallaviNeural',
    speed: 1.02,
    pitch: 0.0,
    cadenceHints: 'Crisp consonants, energetic pace, and authentic Tamil phrase-final cadence',
  },

  // ─── Telugu Dialects ─────────────────────────────────────────────────────────
  {
    id: 'te-in-hyderabad',
    language: 'Telugu',
    langCode: 'te-IN',
    dialect: 'Hyderabad / Andhra Colloquial (తెలుగు)',
    flag: '🇮🇳',
    title: 'Conversational Telugu & Street Expressions',
    description: 'Smooth and musical Telugu speech with popular everyday conversational phrases.',
    sampleText: "మోవా, ఈ న్యూరల్ వాయిస్ అవుట్‌పుట్ కేక పుట్టిస్తుంది రా! అంతా పక్కాగా సెట్ అయిపోయింది, వెంటనే రికార్డింగ్ మొదలుపెట్టేద్దాం.",
    recommendedVoice: 'te-IN-ShrutiNeural',
    speed: 1.0,
    pitch: 0.0,
    cadenceHints: 'Melodic sentence arcs, smooth vowel harmony, and conversational warmth',
  },

  // ─── Kannada Dialects ────────────────────────────────────────────────────────
  {
    id: 'kn-in-bengaluru',
    language: 'Kannada',
    langCode: 'kn-IN',
    dialect: 'Bengaluru Casual & Colloquial (ಕನ್ನಡ)',
    flag: '🇮🇳',
    title: 'Bengaluru Youth & Conversational Kannada',
    description: 'Friendly, fluid Kannada speech with authentic colloquial particles.',
    sampleText: "ಮಗಾ, ಈ ವಾಯ್ಸ್ ಔಟ್‌ಪುಟ್ ಸಕ್ಕತ್ತಾಗಿ ಕೇಳಿಸ್ತಾ ಇದೆ ಕಣೋ! ವರ್ಕ್‌ಫ್ಲೋ ಎಲ್ಲಾ ಕಂಪ್ಲೀಟ್ ಸೆಟ್ ಆಗಿದೆ, ಇವಾಗಲೇ ಲೈವ್ ಹೋಗೋಣ.",
    recommendedVoice: 'kn-IN-SapnaNeural',
    speed: 1.0,
    pitch: 0.0,
    cadenceHints: 'Relaxed tempo, natural pause timing, and friendly native inflection',
  },

  // ─── Italian Dialects ────────────────────────────────────────────────────────
  {
    id: 'it-it-casual',
    language: 'Italian',
    langCode: 'it-IT',
    dialect: 'Italian Conversational Casual',
    flag: '🇮🇹',
    title: 'Italian Colloquial & Exclamations',
    description: 'Warm, expressive Italian with conversational particles and enthusiastic melody.',
    sampleText: "Ma dai, che figata assurda! Guarda, il suono è veramente pazzesco. Ci prendiamo un caffè e finiamo la traccia?",
    recommendedVoice: 'it-IT-ElsaNeural',
    speed: 1.02,
    pitch: 0.2,
    cadenceHints: 'Sing-song melodic contour, prolonged stressed syllables, lively gestures in cadence',
  },

  // ─── Portuguese Dialects ─────────────────────────────────────────────────────
  {
    id: 'pt-br-carioca',
    language: 'Portuguese',
    langCode: 'pt-BR',
    dialect: 'Brazilian (Rio / São Paulo Urban)',
    flag: '🇧🇷',
    title: 'Brazilian Portuguese Casual',
    description: 'Laid-back, musical Brazilian speech with popular street expressions.',
    sampleText: "Fala cara, beleza? Cara, esse áudio ficou muito legal e tranquilo! Valeu mesmo pela força, bora gravar mais.",
    recommendedVoice: 'pt-BR-FranciscaNeural',
    speed: 1.0,
    pitch: 0.0,
    cadenceHints: 'Nasal vowel richness, fluid rhythm, relaxed conversational glide',
  },

  // ─── Arabic Dialects ─────────────────────────────────────────────────────────
  {
    id: 'ar-dialectal',
    language: 'Arabic',
    langCode: 'ar-SA',
    dialect: 'Conversational Levantine / Arab Gulf',
    flag: '🇸🇦',
    title: 'Conversational Colloquial Arabic',
    description: 'Friendly, warm spoken Arabic with common greetings, affirmations, and pacing.',
    sampleText: "يلا يا حبيبي، ما شاء الله الصوت طالع مية مية ومضبوط تمام! يعطيك ألف عافية، خلينا نسمع التسجيل كامل.",
    recommendedVoice: 'ar-SA-ZariyahNeural',
    speed: 1.0,
    pitch: 0.0,
    cadenceHints: 'Warm pharyngeal resonance, rhythmic flow with natural honorific pauses',
  },
];
