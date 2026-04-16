# Sanskrit Devanagari Locale — Design Spec

**Date:** 2026-04-16
**Crate:** `vedaksha-locale`
**Breaking:** Yes — `sa` output changes from IAST Latin to Devanagari script

## Summary

Replace all IAST romanization strings in the `sa` (Sanskrit) locale with
Devanagari script. No new language codes, no new API surface, no IAST
fallback. Pure data replacement across 10 source files.

## Motivation

- IAST is a romanization scheme for academic papers, not a display script
- Users selecting Sanskrit expect Devanagari (the native script)
- All other Indic locales (hi, ta, te, kn, bn) already return native scripts
- MCP's language toggle shows Sanskrit as "संस्कृतम्" — IAST output is inconsistent

## Scope

### Modules affected (10 files)

| Module | Array | Count | Example |
|--------|-------|-------|---------|
| planets.rs | `PLANETS_SA` | 9 | Sūrya → सूर्यः |
| signs.rs | `SIGNS_SA` | 12 | Meṣa → मेषः |
| nakshatras.rs | `NAKSHATRAS_SA` | 27 | Aśvinī → अश्विनी |
| deities.rs | `DEITIES_SA` | 27 | Aśvinau → अश्विनौ |
| houses.rs | `HOUSES_SA` | 12 | Prathama Bhāva → प्रथमभावः |
| dignities.rs | `DIGNITIES_SA` | 5 | Ucca → उच्चम् |
| karanas.rs | `KARANAS_SA` | 11 | Bava → बवः |
| panchanga_yogas.rs | `PANCHANGA_YOGAS_SA` | 27 | Viṣkambha → विष्कम्भः |
| yogas.rs | `YOGAS_SA` | 8 | Gajakesarī → गजकेसरी |
| aspects.rs | `ASPECTS_SA` | 11 | Yuti → युतिः |

`dashas.rs` delegates to `planets.rs` — no own array to change.

### lib.rs change

`Language::Sanskrit.native_name()` changes from `"Saṃskṛtam"` (IAST) to
`"संस्कृतम्"` (Devanagari).

## Sanskrit Orthography Rules

1. **Masculine nouns** get visarga ending: सूर्यः, मेषः, मङ्गलः
2. **Feminine nouns** keep natural ending: अश्विनी, भरणी, कन्या
3. **Neuter nouns** get -म् ending where standard: उच्चम्
4. **Conjuncts over anusvara** for classical Sanskrit: मङ्गल not मंगल
5. **Compound words** are single words: प्रथमभावः not प्रथम भावः
6. **No schwa deletion** — Sanskrit retains inherent vowels unlike Hindi

## Complete Replacement Tables

### Planets (9)

| Index | IAST (current) | Devanagari (new) |
|-------|----------------|------------------|
| 0 | Sūrya | सूर्यः |
| 1 | Candra | चन्द्रः |
| 2 | Maṅgala | मङ्गलः |
| 3 | Budha | बुधः |
| 4 | Bṛhaspati | बृहस्पतिः |
| 5 | Śukra | शुक्रः |
| 6 | Śani | शनिः |
| 7 | Rāhu | राहुः |
| 8 | Ketu | केतुः |

### Signs (12)

| Index | IAST | Devanagari |
|-------|------|------------|
| 0 | Meṣa | मेषः |
| 1 | Vṛṣabha | वृषभः |
| 2 | Mithuna | मिथुनम् |
| 3 | Karka | कर्कः |
| 4 | Siṃha | सिंहः |
| 5 | Kanyā | कन्या |
| 6 | Tulā | तुला |
| 7 | Vṛścika | वृश्चिकः |
| 8 | Dhanus | धनुः |
| 9 | Makara | मकरः |
| 10 | Kumbha | कुम्भः |
| 11 | Mīna | मीनः |

### Nakshatras (27)

| Index | IAST | Devanagari |
|-------|------|------------|
| 0 | Aśvinī | अश्विनी |
| 1 | Bharaṇī | भरणी |
| 2 | Kṛttikā | कृत्तिका |
| 3 | Rohiṇī | रोहिणी |
| 4 | Mṛgaśirā | मृगशिरा |
| 5 | Ārdrā | आर्द्रा |
| 6 | Punarvasu | पुनर्वसु |
| 7 | Puṣya | पुष्यः |
| 8 | Āśleṣā | आश्लेषा |
| 9 | Maghā | मघा |
| 10 | Pūrva Phālgunī | पूर्वफाल्गुनी |
| 11 | Uttara Phālgunī | उत्तरफाल्गुनी |
| 12 | Hasta | हस्तः |
| 13 | Citrā | चित्रा |
| 14 | Svātī | स्वाती |
| 15 | Viśākhā | विशाखा |
| 16 | Anurādhā | अनुराधा |
| 17 | Jyeṣṭhā | ज्येष्ठा |
| 18 | Mūla | मूलम् |
| 19 | Pūrvāṣāḍhā | पूर्वाषाढा |
| 20 | Uttarāṣāḍhā | उत्तराषाढा |
| 21 | Śravaṇa | श्रवणः |
| 22 | Dhaniṣṭhā | धनिष्ठा |
| 23 | Śatabhiṣā | शतभिषा |
| 24 | Pūrva Bhādrapadā | पूर्वभाद्रपदा |
| 25 | Uttara Bhādrapadā | उत्तरभाद्रपदा |
| 26 | Revatī | रेवती |

### Deities (27)

| Index | IAST | Devanagari |
|-------|------|------------|
| 0 | Aśvinau | अश्विनौ |
| 1 | Yama | यमः |
| 2 | Agni | अग्निः |
| 3 | Brahmā | ब्रह्मा |
| 4 | Soma | सोमः |
| 5 | Rudra | रुद्रः |
| 6 | Aditi | अदितिः |
| 7 | Bṛhaspati | बृहस्पतिः |
| 8 | Sarpāḥ | सर्पाः |
| 9 | Pitaraḥ | पितरः |
| 10 | Bhaga | भगः |
| 11 | Aryamā | अर्यमा |
| 12 | Savitṛ | सवितृ |
| 13 | Tvaṣṭṛ | त्वष्टृ |
| 14 | Vāyu | वायुः |
| 15 | Indrāgnī | इन्द्राग्नी |
| 16 | Mitra | मित्रः |
| 17 | Indra | इन्द्रः |
| 18 | Nirṛti | निरृतिः |
| 19 | Āpaḥ | आपः |
| 20 | Viśvedevāḥ | विश्वेदेवाः |
| 21 | Viṣṇu | विष्णुः |
| 22 | Vasavaḥ | वसवः |
| 23 | Varuṇa | वरुणः |
| 24 | Ajaikapāt | अजैकपात् |
| 25 | Ahirbudhnya | अहिर्बुध्न्यः |
| 26 | Pūṣan | पूषन् |

### Houses (12)

| Number | IAST | Devanagari |
|--------|------|------------|
| 1 | Prathama Bhāva | प्रथमभावः |
| 2 | Dvitīya Bhāva | द्वितीयभावः |
| 3 | Tṛtīya Bhāva | तृतीयभावः |
| 4 | Caturtha Bhāva | चतुर्थभावः |
| 5 | Pañcama Bhāva | पञ्चमभावः |
| 6 | Ṣaṣṭha Bhāva | षष्ठभावः |
| 7 | Saptama Bhāva | सप्तमभावः |
| 8 | Aṣṭama Bhāva | अष्टमभावः |
| 9 | Navama Bhāva | नवमभावः |
| 10 | Daśama Bhāva | दशमभावः |
| 11 | Ekādaśa Bhāva | एकादशभावः |
| 12 | Dvādaśa Bhāva | द्वादशभावः |

### Dignities (5)

| Index | IAST | Devanagari |
|-------|------|------------|
| 0 | Ucca | उच्चम् |
| 1 | Mūlatrikoṇa | मूलत्रिकोणम् |
| 2 | Svakṣetra | स्वक्षेत्रम् |
| 3 | Nīca | नीचम् |
| 4 | Śatru | शत्रुक्षेत्रम् |

### Karanas (11)

| Index | IAST | Devanagari |
|-------|------|------------|
| 0 | Bava | बवः |
| 1 | Bālava | बालवः |
| 2 | Kaulava | कौलवः |
| 3 | Taitila | तैतिलः |
| 4 | Gara | गरः |
| 5 | Vaṇij | वणिज् |
| 6 | Viṣṭi | विष्टिः |
| 7 | Śakuni | शकुनिः |
| 8 | Catuṣpāt | चतुष्पात् |
| 9 | Nāga | नागः |
| 10 | Kiṃstughna | किंस्तुघ्नः |

### Panchanga Yogas (27)

| Index | IAST | Devanagari |
|-------|------|------------|
| 0 | Viṣkambha | विष्कम्भः |
| 1 | Prīti | प्रीतिः |
| 2 | Āyuṣmān | आयुष्मान् |
| 3 | Saubhāgya | सौभाग्यम् |
| 4 | Śobhana | शोभनम् |
| 5 | Atigaṇḍa | अतिगण्डः |
| 6 | Sukarma | सुकर्मा |
| 7 | Dhṛti | धृतिः |
| 8 | Śūla | शूलः |
| 9 | Gaṇḍa | गण्डः |
| 10 | Vṛddhi | वृद्धिः |
| 11 | Dhruva | ध्रुवः |
| 12 | Vyāghāta | व्याघातः |
| 13 | Harṣaṇa | हर्षणम् |
| 14 | Vajra | वज्रम् |
| 15 | Siddhi | सिद्धिः |
| 16 | Vyatīpāta | व्यतीपातः |
| 17 | Variyas | वरीयस् |
| 18 | Parigha | परिघः |
| 19 | Śiva | शिवः |
| 20 | Siddha | सिद्धः |
| 21 | Sādhya | साध्यः |
| 22 | Śubha | शुभः |
| 23 | Śukla | शुक्लः |
| 24 | Brahma | ब्रह्मा |
| 25 | Indra | इन्द्रः |
| 26 | Vaidhṛti | वैधृतिः |

### Yogas (8)

| Index | IAST | Devanagari |
|-------|------|------------|
| 0 | Gajakesarī | गजकेसरी |
| 1 | Budha-Āditya | बुधादित्यः |
| 2 | Parivartana | परिवर्तनम् |
| 3 | Rāja | राजयोगः |
| 4 | Dhana | धनयोगः |
| 5 | Mahāpuruṣa | महापुरुषयोगः |
| 6 | Kālasarpa | कालसर्पः |
| 7 | Viparīta Rāja | विपरीतराजयोगः |

### Aspects (11)

| Index | IAST | Devanagari |
|-------|------|------------|
| 0 | Yuti | युतिः |
| 1 | Saptama Dṛṣṭi | सप्तमदृष्टिः |
| 2 | Pañcama Dṛṣṭi | पञ्चमदृष्टिः |
| 3 | Navama Dṛṣṭi | नवमदृष्टिः |
| 4 | Tṛtīya Dṛṣṭi | तृतीयदृष्टिः |
| 5 | Daśama Dṛṣṭi | दशमदृष्टिः |
| 6 | Caturtha Dṛṣṭi | चतुर्थदृष्टिः |
| 7 | Aṣṭama Dṛṣṭi | अष्टमदृष्टिः |
| 8 | Dvādaśa Dṛṣṭi | द्वादशदृष्टिः |
| 9 | Ṣaḍaṣṭaka | षडष्टकम् |
| 10 | Kendra | केन्द्रम् |

## What Does NOT Change

- `Language` enum variants — no additions or removals
- Function signatures — `(index/number, Language) -> &'static str`
- Array sizes — same counts per module
- `no_std` compatibility — UTF-8 static strings
- Module structure — same 12 files

## Tests

- Update `sign_name_sanskrit_aries` assertion: `"Meṣa"` → `"मेषः"`
- Update `planet_name_sanskrit_sun` if exists
- All existing `all_languages_*_non_empty` tests pass unchanged
- Add one Devanagari spot-check test per module (11 new assertions)
