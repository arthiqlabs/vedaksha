# Sanskrit Devanagari Locale — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace all IAST romanization in the `sa` locale with Devanagari script across vedaksha-locale.

**Architecture:** Pure data swap — replace static string arrays in 10 modules, update `native_name()` in lib.rs, fix 2 existing test assertions, add 10 spot-check tests. No API changes.

**Tech Stack:** Rust, `no_std` static string arrays, `cargo test`

**Spec:** `docs/superpowers/specs/2026-04-16-sanskrit-devanagari-locale-design.md`

---

### Task 1: Update native_name and planets

**Files:**
- Modify: `crates/vedaksha-locale/src/lib.rs:70` (native_name)
- Modify: `crates/vedaksha-locale/src/planets.rs` (PLANETS_SA array)

- [ ] **Step 1: Update native_name for Sanskrit**

In `crates/vedaksha-locale/src/lib.rs`, change line 70:

```rust
// old:
Self::Sanskrit => "Saṃskṛtam",
// new:
Self::Sanskrit => "संस्कृतम्",
```

- [ ] **Step 2: Replace PLANETS_SA array**

In `crates/vedaksha-locale/src/planets.rs`, replace the `PLANETS_SA` array with:

```rust
static PLANETS_SA: &[&str] = &[
    "सूर्यः",
    "चन्द्रः",
    "मङ्गलः",
    "बुधः",
    "बृहस्पतिः",
    "शुक्रः",
    "शनिः",
    "राहुः",
    "केतुः",
];
```

- [ ] **Step 3: Run tests**

Run: `cargo test -p vedaksha-locale -- --nocapture 2>&1 | tail -5`
Expected: Some tests fail (existing IAST assertions). Non-empty tests still pass.

- [ ] **Step 4: Commit**

```bash
git add crates/vedaksha-locale/src/lib.rs crates/vedaksha-locale/src/planets.rs
git commit -m "feat(locale): Sanskrit planets + native_name → Devanagari"
```

---

### Task 2: Update signs

**Files:**
- Modify: `crates/vedaksha-locale/src/signs.rs` (SIGNS_SA array)

- [ ] **Step 1: Replace SIGNS_SA array**

In `crates/vedaksha-locale/src/signs.rs`, replace the `SIGNS_SA` array with:

```rust
static SIGNS_SA: &[&str] = &[
    "मेषः",
    "वृषभः",
    "मिथुनम्",
    "कर्कः",
    "सिंहः",
    "कन्या",
    "तुला",
    "वृश्चिकः",
    "धनुः",
    "मकरः",
    "कुम्भः",
    "मीनः",
];
```

- [ ] **Step 2: Commit**

```bash
git add crates/vedaksha-locale/src/signs.rs
git commit -m "feat(locale): Sanskrit signs → Devanagari"
```

---

### Task 3: Update nakshatras

**Files:**
- Modify: `crates/vedaksha-locale/src/nakshatras.rs` (NAKSHATRAS_SA array)

- [ ] **Step 1: Replace NAKSHATRAS_SA array**

In `crates/vedaksha-locale/src/nakshatras.rs`, replace the `NAKSHATRAS_SA` array with:

```rust
static NAKSHATRAS_SA: &[&str] = &[
    "अश्विनी",
    "भरणी",
    "कृत्तिका",
    "रोहिणी",
    "मृगशिरा",
    "आर्द्रा",
    "पुनर्वसु",
    "पुष्यः",
    "आश्लेषा",
    "मघा",
    "पूर्वफाल्गुनी",
    "उत्तरफाल्गुनी",
    "हस्तः",
    "चित्रा",
    "स्वाती",
    "विशाखा",
    "अनुराधा",
    "ज्येष्ठा",
    "मूलम्",
    "पूर्वाषाढा",
    "उत्तराषाढा",
    "श्रवणः",
    "धनिष्ठा",
    "शतभिषा",
    "पूर्वभाद्रपदा",
    "उत्तरभाद्रपदा",
    "रेवती",
];
```

- [ ] **Step 2: Commit**

```bash
git add crates/vedaksha-locale/src/nakshatras.rs
git commit -m "feat(locale): Sanskrit nakshatras → Devanagari"
```

---

### Task 4: Update deities

**Files:**
- Modify: `crates/vedaksha-locale/src/deities.rs` (DEITIES_SA array)

- [ ] **Step 1: Replace DEITIES_SA array**

In `crates/vedaksha-locale/src/deities.rs`, replace the `DEITIES_SA` array with:

```rust
static DEITIES_SA: &[&str] = &[
    "अश्विनौ",
    "यमः",
    "अग्निः",
    "ब्रह्मा",
    "सोमः",
    "रुद्रः",
    "अदितिः",
    "बृहस्पतिः",
    "सर्पाः",
    "पितरः",
    "भगः",
    "अर्यमा",
    "सवितृ",
    "त्वष्टृ",
    "वायुः",
    "इन्द्राग्नी",
    "मित्रः",
    "इन्द्रः",
    "निरृतिः",
    "आपः",
    "विश्वेदेवाः",
    "विष्णुः",
    "वसवः",
    "वरुणः",
    "अजैकपात्",
    "अहिर्बुध्न्यः",
    "पूषन्",
];
```

- [ ] **Step 2: Commit**

```bash
git add crates/vedaksha-locale/src/deities.rs
git commit -m "feat(locale): Sanskrit deities → Devanagari"
```

---

### Task 5: Update houses

**Files:**
- Modify: `crates/vedaksha-locale/src/houses.rs` (HOUSES_SA array)

- [ ] **Step 1: Replace HOUSES_SA array**

In `crates/vedaksha-locale/src/houses.rs`, replace the `HOUSES_SA` array with:

```rust
static HOUSES_SA: &[&str] = &[
    "प्रथमभावः",
    "द्वितीयभावः",
    "तृतीयभावः",
    "चतुर्थभावः",
    "पञ्चमभावः",
    "षष्ठभावः",
    "सप्तमभावः",
    "अष्टमभावः",
    "नवमभावः",
    "दशमभावः",
    "एकादशभावः",
    "द्वादशभावः",
];
```

- [ ] **Step 2: Commit**

```bash
git add crates/vedaksha-locale/src/houses.rs
git commit -m "feat(locale): Sanskrit houses → Devanagari"
```

---

### Task 6: Update dignities, karanas, aspects

**Files:**
- Modify: `crates/vedaksha-locale/src/dignities.rs` (DIGNITIES_SA array)
- Modify: `crates/vedaksha-locale/src/karanas.rs` (KARANAS_SA array)
- Modify: `crates/vedaksha-locale/src/aspects.rs` (ASPECTS_SA array)

- [ ] **Step 1: Replace DIGNITIES_SA array**

In `crates/vedaksha-locale/src/dignities.rs`:

```rust
static DIGNITIES_SA: &[&str] = &[
    "उच्चम्",
    "मूलत्रिकोणम्",
    "स्वक्षेत्रम्",
    "नीचम्",
    "शत्रुक्षेत्रम्",
];
```

- [ ] **Step 2: Replace KARANAS_SA array**

In `crates/vedaksha-locale/src/karanas.rs`:

```rust
static KARANAS_SA: &[&str] = &[
    "बवः",
    "बालवः",
    "कौलवः",
    "तैतिलः",
    "गरः",
    "वणिज्",
    "विष्टिः",
    "शकुनिः",
    "चतुष्पात्",
    "नागः",
    "किंस्तुघ्नः",
];
```

- [ ] **Step 3: Replace ASPECTS_SA array**

In `crates/vedaksha-locale/src/aspects.rs`:

```rust
static ASPECTS_SA: &[&str] = &[
    "युतिः",
    "सप्तमदृष्टिः",
    "पञ्चमदृष्टिः",
    "नवमदृष्टिः",
    "तृतीयदृष्टिः",
    "दशमदृष्टिः",
    "चतुर्थदृष्टिः",
    "अष्टमदृष्टिः",
    "द्वादशदृष्टिः",
    "षडष्टकम्",
    "केन्द्रम्",
];
```

- [ ] **Step 4: Commit**

```bash
git add crates/vedaksha-locale/src/dignities.rs crates/vedaksha-locale/src/karanas.rs crates/vedaksha-locale/src/aspects.rs
git commit -m "feat(locale): Sanskrit dignities, karanas, aspects → Devanagari"
```

---

### Task 7: Update panchanga_yogas and yogas

**Files:**
- Modify: `crates/vedaksha-locale/src/panchanga_yogas.rs` (PANCHANGA_YOGAS_SA array)
- Modify: `crates/vedaksha-locale/src/yogas.rs` (YOGAS_SA array)

- [ ] **Step 1: Replace PANCHANGA_YOGAS_SA array**

In `crates/vedaksha-locale/src/panchanga_yogas.rs`:

```rust
static PANCHANGA_YOGAS_SA: &[&str] = &[
    "विष्कम्भः",
    "प्रीतिः",
    "आयुष्मान्",
    "सौभाग्यम्",
    "शोभनम्",
    "अतिगण्डः",
    "सुकर्मा",
    "धृतिः",
    "शूलः",
    "गण्डः",
    "वृद्धिः",
    "ध्रुवः",
    "व्याघातः",
    "हर्षणम्",
    "वज्रम्",
    "सिद्धिः",
    "व्यतीपातः",
    "वरीयस्",
    "परिघः",
    "शिवः",
    "सिद्धः",
    "साध्यः",
    "शुभः",
    "शुक्लः",
    "ब्रह्मा",
    "इन्द्रः",
    "वैधृतिः",
];
```

- [ ] **Step 2: Replace YOGAS_SA array**

In `crates/vedaksha-locale/src/yogas.rs`:

```rust
static YOGAS_SA: &[&str] = &[
    "गजकेसरी",
    "बुधादित्यः",
    "परिवर्तनम्",
    "राजयोगः",
    "धनयोगः",
    "महापुरुषयोगः",
    "कालसर्पः",
    "विपरीतराजयोगः",
];
```

- [ ] **Step 3: Commit**

```bash
git add crates/vedaksha-locale/src/panchanga_yogas.rs crates/vedaksha-locale/src/yogas.rs
git commit -m "feat(locale): Sanskrit panchanga yogas, yogas → Devanagari"
```

---

### Task 8: Fix existing tests and add spot-checks

**Files:**
- Modify: `crates/vedaksha-locale/src/lib.rs` (test assertions)

- [ ] **Step 1: Fix existing Sanskrit test assertions**

In `crates/vedaksha-locale/src/lib.rs`, update these two tests:

Line 159-161 — change:
```rust
fn sign_name_sanskrit_aries() {
    assert_eq!(signs::sign_name(0, Language::Sanskrit), "Meṣa");
}
```
to:
```rust
fn sign_name_sanskrit_aries() {
    assert_eq!(signs::sign_name(0, Language::Sanskrit), "मेषः");
}
```

Line 305-307 — change:
```rust
fn karana_name_sanskrit_vishti() {
    assert_eq!(karanas::karana_name(6, Language::Sanskrit), "Viṣṭi");
}
```
to:
```rust
fn karana_name_sanskrit_vishti() {
    assert_eq!(karanas::karana_name(6, Language::Sanskrit), "विष्टिः");
}
```

- [ ] **Step 2: Add Devanagari spot-check tests**

Add these tests in `crates/vedaksha-locale/src/lib.rs` after the existing Sanskrit tests (around line 310):

```rust
#[test]
fn sanskrit_planet_sun_devanagari() {
    assert_eq!(planets::planet_name(0, Language::Sanskrit), "सूर्यः");
}

#[test]
fn sanskrit_nakshatra_ashwini_devanagari() {
    assert_eq!(nakshatras::nakshatra_name(0, Language::Sanskrit), "अश्विनी");
}

#[test]
fn sanskrit_deity_yama_devanagari() {
    assert_eq!(deities::deity_name(1, Language::Sanskrit), "यमः");
}

#[test]
fn sanskrit_house_first_devanagari() {
    assert_eq!(houses::house_name(1, Language::Sanskrit), "प्रथमभावः");
}

#[test]
fn sanskrit_dignity_ucca_devanagari() {
    assert_eq!(dignities::dignity_name(0, Language::Sanskrit), "उच्चम्");
}

#[test]
fn sanskrit_panchanga_yoga_vishkambha_devanagari() {
    assert_eq!(panchanga_yogas::panchanga_yoga_name(0, Language::Sanskrit), "विष्कम्भः");
}

#[test]
fn sanskrit_yoga_gajakesari_devanagari() {
    assert_eq!(yogas::yoga_name(0, Language::Sanskrit), "गजकेसरी");
}

#[test]
fn sanskrit_aspect_yuti_devanagari() {
    assert_eq!(aspects::aspect_name(0, Language::Sanskrit), "युतिः");
}

#[test]
fn sanskrit_native_name_devanagari() {
    assert_eq!(Language::Sanskrit.native_name(), "संस्कृतम्");
}
```

- [ ] **Step 3: Run full test suite**

Run: `cargo test -p vedaksha-locale`
Expected: ALL tests pass — 0 failures.

- [ ] **Step 4: Run clippy**

Run: `cargo clippy -p vedaksha-locale -- -D warnings`
Expected: 0 errors.

- [ ] **Step 5: Commit**

```bash
git add crates/vedaksha-locale/src/lib.rs
git commit -m "test(locale): fix Sanskrit assertions, add Devanagari spot-checks"
```
