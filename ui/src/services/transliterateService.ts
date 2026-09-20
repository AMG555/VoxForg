/**
 * VoxForg Transliteration & Phonetic Normalizer Service
 * Provides bidirectional & forward transliteration for Romanized Indic dialects:
 * - Manglish (Malayalam written in Latin script) -> Malayalam Script (മലയാളം)
 * - Hinglish (Hindi written in Latin script) -> Devanagari Script (हिन्दी)
 * - Tanglish (Tamil written in Latin script) -> Tamil Script (தமிழ்)
 */
// Multi-word phrase mappings have highest precedence
const MANGLISH_PHRASES: [RegExp, string][] = [
  [/\benthokke\s+undu?\s+vishesham\b/gi, 'എന്തൊക്കെയുണ്ട് വിശേഷം'],
  [/\benthokke\s+undu?\b/gi, 'എന്തൊക്കെയുണ്ട്'],
  [/\boru\s+rakshayum\s+illa\b/gi, 'ഒരു രക്ഷയുമില്ല'],
  [/\bscene\s+aanu\b/gi, 'സീൻ ആണ്'],
  [/\bset\s+aayi\b/gi, 'സെറ്റ് ആയി'],
  [/\bente\s+ponno\b/gi, 'എന്റെ പൊന്നോ'],
  [/\bvalare\s+nannayi\b/gi, 'വളരെ നന്നായി'],
  [/\bithu\s+adipoli\s+aano\b/gi, 'ഇത് അടിപൊളി ആണോ'],
  [/\bvoxforg\s+studio\b/gi, 'വോക്സ്ഫോർജ് സ്റ്റുഡിയോ'],
  [/\bparayan\s+pattumo\b/gi, 'പറയാൻ പറ്റുമോ'],
  [/\bnjan\s+ippo\s+varam\b/gi, 'ഞാൻ ഇപ്പോൾ വരാം'],
];

const MANGLISH_WORDS: Record<string, string> = {
  'adipoli': 'അടിപൊളി',
  'adpoli': 'അടിപൊളി',
  'kidilam': 'കിടിലം',
  'machane': 'മച്ചാനേ',
  'chunke': 'ചങ്കേ',
  'pwoli': 'പൊളി',
  'poli': 'പൊളി',
  'polichu': 'പൊളിച്ചു',
  'vishesham': 'വിശേഷം',
  'sukhamano': 'സുഖമാണോ',
  'namaskaram': 'നമസ്കാരം',
  'njan': 'ഞാൻ',
  'njangal': 'ഞങ്ങൾ',
  'ninte': 'നിന്റെ',
  'ningal': 'നിങ്ങൾ',
  'ivide': 'ഇവിടെ',
  'avide': 'അവിടെ',
  'evide': 'എവിടെ',
  'evideya': 'എവിടെ',
  'poda': 'പോടാ',
  'mone': 'മോനേ',
  'chetta': 'ചേട്ടാ',
  'chechi': 'ചേച്ചീ',
  'aano': 'ആണോ',
  'aanu': 'ആണ്',
  'und': 'ഉണ്ട്',
  'undu': 'ഉണ്ട്',
  'illa': 'ഇല്ല',
  'valare': 'വളരെ',
  'nannayi': 'നന്നായി',
  'manassilayi': 'മനസ്സിലായി',
  'engane': 'എങ്ങനെ',
  'ippo': 'ഇപ്പോൾ',
  'ippol': 'ഇപ്പോൾ',
  'varam': 'വരാം',
  'kollam': 'കൊള്ളാം',
  'kollaam': 'കൊള്ളാം',
  'kollalo': 'കൊള്ളാലോ',
  'thanne': 'തന്നെ',
  'sathyam': 'സത്യം',
  'pinne': 'പിന്നെ',
  'nokku': 'നോക്കൂ',
  'parayu': 'പറയൂ',
  'parayam': 'പറയാം',
  'parayamo': 'പറയാമോ',
  'enthaanu': 'എന്താണ്',
  'entha': 'എന്ത്',
  'kidu': 'കിടു',
  'mass': 'മാസ്സ്',
  'chumma': 'ചുമ്മാ',
  'ariyilla': 'അറിയില്ല',
  'sheriyaanu': 'ശരിയാണ്',
  'sheriyaa': 'ശരിയാ',
  'veruthe': 'വെറുതെ',
  'thante': 'തന്റെ',
  'cheyyanam': 'ചെയ്യണം',
  'enthoru': 'എന്തൊരു',
  'nalla': 'നല്ല',
  'thannathaane': 'തന്നത്താനെ',
  'kaaryam': 'കാര്യം',
  'oru': 'ഒരു',
  'ithu': 'ഇത്',
  'athu': 'അത്',
  'nammude': 'നമ്മുടെ',
  'naale': 'നാളെ',
  'innu': 'ഇന്ന്',
  'innale': 'ഇന്നലെ',
  'varumo': 'വരുമോ',
  'poyalo': 'പോയാലോ',
  'kelkkamo': 'കേൾക്കാമോ',
  'aliya': 'അളിയാ',
  'aliyan': 'അളിയൻ',
  'dhaivame': 'ദൈവമേ',
  'ente': 'എന്റെ',
  'ponno': 'പൊന്നോ',
  'bro': 'ബ്രോ',
  'scene': 'സീൻ',
  'sound': 'സൗണ്ട്',
  'voice': 'വോയ്സ്',
  'output': 'ഔട്ട്പുട്ട്',
  'voxforg': 'വോക്സ്ഫോർജ്',
  'studio': 'സ്റ്റുഡിയോ',
  'santhosham': 'സന്തോഷം',
  'kandathil': 'കണ്ടതിൽ',
  'swagatham': 'സ്വാഗതം',
  'nandi': 'നന്ദി',
  'parayunnu': 'പറയുന്നു',
  'paranjathu': 'പറഞ്ഞത്',
  'ariyaam': 'അറിയാം',
  'venam': 'വേണം',
  'venda': 'വേണ്ട',
  'sahayam': 'സഹായം',
  'sahayikkamo': 'സഹായിക്കാമോ',
  'karyam': 'കാര്യം',
  'karyangal': 'കാര്യങ്ങൾ',
  'nammal': 'നമ്മൾ',
  'samayam': 'സമയം',
  'pettannu': 'പെട്ടെന്ന്',
  'theerchayayum': 'തീർച്ചയായും',
  'thettanu': 'തെറ്റാണ്',
  'sathyamanu': 'സത്യമാണ്',
  'kooduthal': 'കൂടുതൽ',
  'kuravu': 'കുറവ്',
  'dharalam': 'ധാരാളം',
  'eppozhum': 'എപ്പോഴും',
  'ippozhum': 'ഇപ്പോഴും',
  'ethrayum': 'എത്രയും',
  'ithrayum': 'ഇത്രയും',
  'athrayum': 'അത്രയും',
  'varunnu': 'വരുന്നു',
  'pokunnu': 'പോകുന്നു',
  'vanna': 'വന്ന',
  'poya': 'പോയ',
  'cheythu': 'ചെയ്തു',
  'cheyyunnu': 'ചെയ്യുന്നു',
  'puthiya': 'പുതിയ',
  'pazhaya': 'പഴയ',
  'valiya': 'വലിയ',
  'cheriya': 'ചെറിയ',
  'athra': 'അത്ര',
  'ithra': 'ഇത്ര',
  'ethra': 'എത്ര',
  'aaru': 'ആര്',
  'aaranu': 'ആരാണ്',
  'enthanu': 'എന്താണ്',
  'enganeanu': 'എങ്ങനെയാണ്',
  'evideyanu': 'എവിടെയാണ്',
  'ithaanu': 'ഇതാണ്',
  'athaanu': 'അതാണ്',
  'ithokke': 'ഇതൊക്കെ',
  'athokke': 'അതൊക്കെ',
  'enthellam': 'എന്തെല്ലാം',
  'ellam': 'എല്ലാം',
  'ellavarkkum': 'എല്ലാവർക്കും',
  'ellavarum': 'എല്ലാവരും',
  'mathram': 'മാത്രം',
  'koodi': 'കൂടി',
  'aanallo': 'ആണല്ലോ',
  'undallo': 'ഉണ്ടല്ലോ',
  'illallo': 'ഇല്ലല്ലോ',
  'veendum': 'വീണ്ടും',
  'neram': 'നേരം',
};

const HINGLISH_PHRASES: [RegExp, string][] = [
  [/\bkya\s+haal\s+hai\s+bhai\b/gi, 'क्या हाल है भाई'],
  [/\bkya\s+haal\s+hai\b/gi, 'क्या हाल है'],
  [/\bekdum\s+mast\b/gi, 'एकदम मस्त'],
  [/\bbahut\s+badhiya\b/gi, 'बहुत बढ़िया'],
  [/\btheek\s+hai\b/gi, 'ठीक है'],
  [/\bkuch\s+nahin?\b/gi, 'कुछ नहीं'],
  [/\bkaise\s+ho\b/gi, 'कैसे हो'],
  [/\bkahan\s+ho\b/gi, 'कहाँ हो'],
  [/\bvoxforg\s+studio\b/gi, 'वॉक्सफोर्ज स्टूडियो'],
];

const HINGLISH_WORDS: Record<string, string> = {
  'bhai': 'भाई',
  'kya': 'क्या',
  'haal': 'हाल',
  'chal': 'चाल',
  'ekdum': 'एकदम',
  'mast': 'मस्त',
  'achha': 'अच्छा',
  'accha': 'अच्छा',
  'theek': 'ठीक',
  'shukriya': 'शुक्रिया',
  'dhanyawad': 'धन्यवाद',
  'yaar': 'यार',
  'kaise': 'कैसे',
  'kahan': 'कहाँ',
  'karo': 'करो',
  'suno': 'सुनो',
  'dekho': 'देखो',
  'kuch': 'कुछ',
  'nahi': 'नहीं',
  'nahin': 'नहीं',
  'hota': 'होता',
  'samajh': 'समझ',
  'gaya': 'गया',
  'chalega': 'चलेगा',
  'bahut': 'बहुत',
  'badhiya': 'बढ़िया',
  'jhakaas': 'झकास',
  'arre': 'अरे',
  'arey': 'अरे',
  'bolo': 'बोलो',
  'apna': 'अपना',
  'apne': 'अपने',
  'kaam': 'काम',
  'karna': 'करना',
  'kardo': 'करदो',
  'zara': 'ज़रा',
  'sunao': 'सुनाओ',
  'sabko': 'सबको',
  'hoga': 'होगा',
  'namaste': 'नमस्ते',
  'dost': 'दोस्त',
  'chalo': 'चलो',
  'batao': 'बताओ',
  'sahi': 'सही',
  'hai': 'है',
  'hain': 'हैं',
  'mujhe': 'मुझे',
  'tumhe': 'तुम्हें',
  'hum': 'हम',
  'aap': 'आप',
  'mera': 'मेरा',
  'tera': 'तेरा',
  'voxforg': 'वॉक्सफोर्ज',
  'studio': 'स्टूडियो',
};

const TANGLISH_PHRASES: [RegExp, string][] = [
  [/\bepdi\s+irukka\b/gi, 'எப்படி இருக்க'],
  [/\bvera\s+level\b/gi, 'வேற லெவல்'],
  [/\bvanakkam\s+machi\b/gi, 'வணக்கம் மச்சி'],
  [/\bsemma\s+sound\b/gi, 'செம்ம சவுண்ட்'],
];

const TANGLISH_WORDS: Record<string, string> = {
  'vanakkam': 'வணக்கம்',
  'machi': 'மச்சி',
  'machan': 'மச்சான்',
  'semma': 'செம்ம',
  'level': 'லெவல்',
  'nandri': 'நன்றி',
  'supera': 'சூப்பரா',
  'romba': 'ரொம்ப',
  'thalaiva': 'தலைவா',
  'kandippa': 'கண்டிப்பா',
  'seri': 'சரி',
  'nanba': 'நண்பா',
  'nanbi': 'நண்பி',
  'enna': 'என்ன',
  'inga': 'இங்க',
  'anga': 'அங்க',
  'voxforg': 'வாக்ஸ்ஃபோர்ஜ்',
};

function phoneticFallbackMalayalam(rawWord: string): string {
  const word = rawWord.toLowerCase();
  const CONSONANTS: [string, string][] = [
    ['zh', 'ഴ'], ['ch', 'ച'], ['th', 'ത'], ['sh', 'ഷ'], ['kh', 'ഖ'],
    ['gh', 'ഘ'], ['jh', 'ഝ'], ['dh', 'ധ'], ['bh', 'ഭ'], ['ph', 'ഫ'],
    ['ng', 'ങ'], ['nj', 'ഞ'], ['k', 'ക'], ['g', 'ഗ'], ['j', 'ജ'],
    ['t', 'ട'], ['d', 'ഡ'], ['n', 'ന'], ['p', 'പ'], ['b', 'ബ'],
    ['m', 'മ'], ['y', 'യ'], ['r', 'ര'], ['l', 'ല'], ['v', 'വ'],
    ['w', 'വ'], ['s', 'സ'], ['h', 'ഹ'],
  ];

  const VOWEL_SIGNS: [string, string][] = [
    ['aa', 'ാ'], ['ee', 'ീ'], ['oo', 'ൂ'], ['ai', 'ൈ'], ['au', 'ൌ'],
    ['ou', 'ൌ'], ['a', ''], ['i', 'ി'], ['u', 'ു'], ['e', 'െ'], ['o', 'ൊ'],
  ];

  const INDEPENDENT_VOWELS: [string, string][] = [
    ['aa', 'ആ'], ['ee', 'ഈ'], ['oo', 'ഊ'], ['ai', 'ഐ'], ['au', 'ഔ'],
    ['ou', 'ഔ'], ['a', 'അ'], ['i', 'ഇ'], ['u', 'ഉ'], ['e', 'എ'], ['o', 'ഒ'],
  ];

  let res = '';
  let i = 0;
  while (i < word.length) {
    if (i === 0) {
      let matchedVowel = false;
      for (const [vStr, vMal] of INDEPENDENT_VOWELS) {
        if (word.startsWith(vStr, i)) {
          res += vMal;
          i += vStr.length;
          matchedVowel = true;
          break;
        }
      }
      if (matchedVowel) continue;
    }

    let matchedConsonant = false;
    for (const [cStr, cMal] of CONSONANTS) {
      if (word.startsWith(cStr, i)) {
        res += cMal;
        i += cStr.length;
        matchedConsonant = true;

        let matchedSign = false;
        for (const [vStr, vSign] of VOWEL_SIGNS) {
          if (word.startsWith(vStr, i)) {
            res += vSign;
            i += vStr.length;
            matchedSign = true;
            break;
          }
        }
        if (!matchedSign) {
          if (i >= word.length || !'aeiou'.includes(word[i])) {
            res += '്';
          }
        }
        break;
      }
    }
    if (matchedConsonant) continue;

    res += word[i];
    i++;
  }
  return res;
}

/**
 * Transliterate Manglish text into native Malayalam Unicode script.
 */
export function transliterateManglishToMalayalam(text: string): string {
  let result = text;

  // 1. Apply multi-word conversational phrases
  for (const [pattern, replacement] of MANGLISH_PHRASES) {
    result = result.replace(pattern, replacement);
  }

  // 2. Tokenize and replace individual vocabulary words while preserving punctuation
  result = result.replace(/\b[A-Za-z']+\b/g, (match) => {
    const lower = match.toLowerCase();
    if (MANGLISH_WORDS[lower]) {
      return MANGLISH_WORDS[lower];
    }
    if (match.length > 2) {
      return phoneticFallbackMalayalam(match);
    }
    return match;
  });

  return result;
}

/**
 * Transliterate Hinglish text into Devanagari script.
 */
export function transliterateHinglishToHindi(text: string): string {
  let result = text;

  for (const [pattern, replacement] of HINGLISH_PHRASES) {
    result = result.replace(pattern, replacement);
  }

  result = result.replace(/\b[A-Za-z']+\b/g, (match) => {
    const lower = match.toLowerCase();
    if (HINGLISH_WORDS[lower]) {
      return HINGLISH_WORDS[lower];
    }
    return match;
  });

  return result;
}

/**
 * Transliterate Tanglish text into Tamil script.
 */
export function transliterateTanglishToTamil(text: string): string {
  let result = text;

  for (const [pattern, replacement] of TANGLISH_PHRASES) {
    result = result.replace(pattern, replacement);
  }

  result = result.replace(/\b[A-Za-z']+\b/g, (match) => {
    const lower = match.toLowerCase();
    if (TANGLISH_WORDS[lower]) {
      return TANGLISH_WORDS[lower];
    }
    return match;
  });

  return result;
}

/**
 * Universal dispatcher to convert Romanized dialect to its respective native script.
 */
export function transliterateToNativeScript(text: string, langCode: string): string {
  switch (langCode) {
    case 'ml-Latn':
    case 'ml-IN':
      return transliterateManglishToMalayalam(text);
    case 'hi-Latn':
    case 'hi-IN':
      return transliterateHinglishToHindi(text);
    case 'ta-Latn':
    case 'ta-IN':
      return transliterateTanglishToTamil(text);
    default:
      return text;
  }
}
