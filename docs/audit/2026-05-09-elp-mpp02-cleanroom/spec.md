# ELP/MPP02 Clean-Room Re-derivation Spec

**Date:** 2026-05-09
**Author:** SPEC subagent (Vedākṣha clean-room re-derivation)
**Status:** Authoritative input for the IMPLEMENTATION subagent. This document is
the **only** ELP/MPP02 reference the implementer is permitted to read.
**Sources cited here are exclusively primary** (IMCCE distribution and the
Chapront & Francou 2003 A&A paper). No third-party implementation has been
consulted.

---

## 0. Provenance summary

- **Path taken:** **Path A** (LLR-fit constants located in IMCCE primary).
  The IMCCE explanatory note `elpmpp02.pdf` (Chapront & Francou, October 2002,
  released with the December 2002 distribution) gives the LLR-fit and DE405-fit
  correction tables directly (Table 3). The FORTRAN reference subroutine
  `ELPMPP02.for` from the same distribution embeds the same numeric corrections
  and is available for cross-check. There is no need to fall back to deriving
  constants from the 2003 A&A paper alone.

- **Primary publication:**
  Chapront J., Francou G., 2003, "The lunar theory ELP revisited. Introduction
  of new planetary perturbations", *Astronomy & Astrophysics* 404, 735.
  DOI: `10.1051/0004-6361:20030529`. Resolved 2026-05-09 to
  `https://doi.org/10.1051/0004-6361:20030529`. The journal page enumerated by
  the DOI confirms title, authors, and abstract; this is the canonical
  paper-of-record cited in the IMCCE README and explanatory note.

- **Authoritative document for the implementer:**
  IMCCE explanatory note `elpmpp02.pdf` ("LUNAR SOLUTION ELP — version
  ELP/MPP02", J. Chapront and G. Francou, Observatoire de Paris — SYRTE,
  October 2002). This 11-page note is self-contained for the construction
  formulas, arguments, frames, and check values; equation/table numbers below
  refer to this note unless otherwise stated.

---

## 1. Frame conventions

### 1.1 Natural ELP frame

The natural coordinate system of the ELP solutions is the **inertial mean
ecliptic of date** with origin at the **departure point γ′₂₀₀₀** defined by
N γ′₂₀₀₀ = N γ_I_₂₀₀₀, where γ_I_₂₀₀₀ is the inertial mean equinox of J2000
and N is the node of the inertial mean ecliptic of date on the inertial mean
ecliptic of J2000. (elpmpp02.pdf §5.1, Fig. 1.)

In this natural frame the geocentric Moon coordinates returned by the ELP/MPP02
series are:

```
V (longitude) = [Σ ELP_MAIN.S1 + Σ ELP_PERT.S1] + W1
U (latitude)  = [Σ ELP_MAIN.S2 + Σ ELP_PERT.S2]
r (distance)  = [Σ ELP_MAIN.S3 + Σ ELP_PERT.S3] · ra0
```
with

```
ra0 = a0(DE405) / a0(ELP) = 384747.961370173 / 384747.980674318
```
(elpmpp02.pdf §5.1, eq. unnumbered between Fig. 1 and the precession block.)

### 1.2 Mean ecliptic of date

To express longitude/latitude on the inertial mean ecliptic and equinox of
**date**:

```
V_d = V + p_A + Δp · t
U_d = U
```
with `p_A` = accumulated precession of Laskar (1986) truncated as

```
p_A = 5029.0966″ t + 1.1120″ t² + 0.000077″ t³ − 0.00002353″ t⁴
Δp  = −0.29965″/cy   (Herring et al., 2002 correction)
```
`t` is barycentric time TDB in Julian centuries from J2000 = JD 2 451 545.0.
(elpmpp02.pdf §5.1.)

### 1.3 Rectangular J2000 ecliptic frame (the canonical output)

The geocentric rectangular coordinates referred to the **inertial mean ecliptic
and equinox of J2000** are obtained by the orthogonal Laskar P/Q rotation from
the (V, U, r) on the ecliptic-of-date:

```
| x_2000^E |   | 1 − 2P²              2PQ                  2P√(1 − P² − Q²)         | | r cos V cos U |
| y_2000^E | = | 2PQ                  1 − 2Q²             −2Q√(1 − P² − Q²)         | | r sin V cos U |
| z_2000^E |   |−2P√(1 − P² − Q²)     2Q√(1 − P² − Q²)    1 − 2P² − 2Q²             | | r sin U       |
```
(elpmpp02.pdf §5.1, eq. unnumbered; matrix as printed.)

`P` and `Q` are issued from Laskar (1986), reproduced in elpmpp02.pdf §5.1
to degree five:

```
P =  0.10180391 × 10⁻⁴ t  + 0.47020439 × 10⁻⁶ t²  − 0.5417367 × 10⁻⁹ t³
    − 0.2507948 × 10⁻¹¹ t⁴ + 0.463486   × 10⁻¹⁴ t⁵

Q = −0.113469002 × 10⁻³ t + 0.12372674 × 10⁻⁶ t²  + 0.1265417 × 10⁻⁸ t³
    − 0.1371808  × 10⁻¹¹ t⁴ − 0.320334   × 10⁻¹⁴ t⁵
```

### 1.4 Units and time scale

- Distance unit: kilometre. Velocity unit: kilometre/day.
  (elpmpp02.pdf §6, "OUTPUT" of `ELPMPP02`.)
- Coefficients `A_{i}` in `ELP_MAIN.S1` and `ELP_MAIN.S2` are arcseconds;
  in `ELP_MAIN.S3` they are kilometres. (elpmpp02.pdf §2.2.1.)
- Coefficients `S_{i}`, `C_{i}` of `ELP_PERT.S{1,2,3}` are arcseconds for
  longitude/latitude and kilometres for distance. (elpmpp02.pdf §2.2.2.)
- Time scale: **TDB**, expressed as Julian centuries from J2000 (denoted `t`)
  for argument polynomials; or as days from J2000 (denoted `tj`) for the
  Fortran subroutine input. (elpmpp02.pdf §3.1, §6.)

> Note on time scale precision: the explanatory note uses TDB throughout. For
> the implementer the practical difference TDB−TT ≤ 2 ms over the validity
> interval translates to lunar position differences far below the millimetre
> level; using TT for `t` is acceptable for any application not chasing
> sub-centimetre LLR-level residuals. The implementer must document whatever
> choice is made; the canonical clean-room implementation should follow the
> note and use TDB.

---

## 2. Series structure (file format and evaluation formulas)

### 2.1 Main-problem files (`ELP_MAIN.S1`, `S2`, `S3`)

**Fourier (purely periodic) series.** First record holds the title and number
of terms; each subsequent record is one periodic term. (elpmpp02.pdf §2.2.1.)

Mathematical form (elpmpp02.pdf eq. (1)):

```
Σ_{i} A_{i} · {sin or cos}( i₁ D + i₂ F + i₃ l + i₄ l′ ),   {i} = (i₁, i₂, i₃, i₄)
```

Per-coordinate trig kernel (elpmpp02.pdf §2.2.1):
- `ELP_MAIN.S1` (Longitude): **sine** series, `A` in arcsec.
- `ELP_MAIN.S2` (Latitude):  **sine** series, `A` in arcsec.
- `ELP_MAIN.S3` (Distance):  **cosine** series, `A` in km.

Record format (FORTRAN): `4i3, 2x, f13.5, 6f12.2`, providing
`i₁ i₂ i₃ i₄  A_{i}  B_{1}^{[i]} B_{2}^{[i]} B_{3}^{[i]} B_{4}^{[i]} B_{5}^{[i]} B_{6}^{[i]}`.

The six `B_j^{[i]}` are partial derivatives of `A_{i}` with respect to the
six constants `σ_j = (m, Γ, E, e′, α, μ)`, dimensionless:

```
B_j = ∂A_{i} / ∂σ_j           (for longitude and latitude)
B_j = (1/a₀) · ∂A_{i} / ∂σ_j  (for distance)
```
(elpmpp02.pdf §2.2.1.)

### 2.2 Perturbation files (`ELP_PERT.S1`, `S2`, `S3`)

**Poisson series.** Records grouped by time power `n ∈ {0, 1, 2, 3, 4}`. Each
group is preceded by a header record (title + count). For a given `n`:

```
t^n · Σ_{i} [ S_{i} sin φ + C_{i} cos φ ],
{i} = (i₁, i₂, i₃, i₄, i₅, i₆, i₇, i₈, i₉, i₁₀, i₁₁, i₁₂, i₁₃)
```
(elpmpp02.pdf eq. (2).)

The phase φ uses Delaunay arguments and planetary arguments:

```
φ = i₁ D + i₂ F + i₃ l + i₄ l′
   + i₅ Me + i₆ V + i₇ T + i₈ Ma + i₉ J + i₁₀ S + i₁₁ U + i₁₂ N
   + i₁₃ ζ
```
(elpmpp02.pdf §2.2.2.)

Record format (FORTRAN): `5x, 2d20.13, 13i3`, providing
`S_{i}  C_{i}  i₁..i₁₃`. Coefficient units: arcsec for S1/S2, km for S3.

### 2.3 Combined evaluation (positions)

```
V(t) = W₁(t)
     + Σ_{Main S1 terms} A_{i} sin(i₁ D + i₂ F + i₃ l + i₄ l′)            [arcsec]
     + Σ_{n=0..4} t^n · Σ_{Pert S1 terms n} [ S_{i} sin φ + C_{i} cos φ ] [arcsec]

U(t) = Σ_{Main S2 terms} A_{i} sin(i₁ D + i₂ F + i₃ l + i₄ l′)            [arcsec]
     + Σ_{n=0..4} t^n · Σ_{Pert S2 terms n} [ S_{i} sin φ + C_{i} cos φ ] [arcsec]

r(t) = ra0 · ( Σ_{Main S3 terms} A_{i} cos(i₁ D + i₂ F + i₃ l + i₄ l′)
            + Σ_{n=0..4} t^n · Σ_{Pert S3 terms n} [ S_{i} sin φ + C_{i} cos φ ] ) [km]
```

Implementation arithmetic note: arcseconds returned by V and U must be reduced
to radians before being used in the trig kernels of §1.3 (e.g. multiply by
`π / (180 · 3600)`).

### 2.4 Combined evaluation (velocities)

The Fortran reference subroutine returns positions and velocities. The
explanatory note (§6) does not print closed-form velocity expressions, so the
implementer must derive them by analytic differentiation of the same series:

```
dV/dt =  W₁'(t)
       + Σ_{Main S1} A_{i} · ω_{i} · cos( i₁ D + i₂ F + i₃ l + i₄ l′ )
       + Σ_{n=0..4} [ n · t^{n-1} · Σ_{Pert S1 n}( S sin φ + C cos φ )
                    +     t^{n}     · Σ_{Pert S1 n}( S · ω_φ · cos φ − C · ω_φ · sin φ ) ]

dU/dt =  same shape as dV/dt without the W₁'(t) leading term and using S2 series.

dr/dt = ra0 · ( Σ_{Main S3} A_{i} · (−ω_{i}) · sin(...)
              + Σ_{n=0..4} [ n · t^{n-1} · Σ_{Pert S3 n}( S sin φ + C cos φ )
                           +     t^{n}     · Σ_{Pert S3 n}( S · ω_φ · cos φ − C · ω_φ · sin φ ) ] )
```

with frequency

```
ω_{i}  = i₁ Ḋ + i₂ Ḟ + i₃ l̇ + i₄ l̇′                                     (main problem)
ω_φ   = i₁ Ḋ + ... + i₄ l̇′ + i₅ Me' + ... + i₁₂ Ṅ + i₁₃ ζ̇                 (perturbations)
```
where `Ẋ` is `dX/dt` of the secular polynomial of §3 (cy⁻¹). To obtain
km/day output velocities the implementer multiplies by `1 / (36525 days/cy)`
inside the Cartesian transform (and converts arcsec/cy to rad/cy on
longitude/latitude rates). Tier-2 velocity check values from elpmpp02.pdf
Tables 8.a apply (see §7).

---

## 3. Fundamental and planetary arguments

### 3.1 Delaunay arguments

(elpmpp02.pdf §3.1, eq. (3).)

```
D  = W₁ − T + 180°
F  = W₁ − W₃
l  = W₁ − W₂
l′ = T − ϖ′
```

W₁, W₂, W₃ are angles of the inertial mean ecliptic of date referred to the
departure point γ′₂₀₀₀. T and ϖ′ are angles of the inertial mean ecliptic of
J2000 referred to the inertial mean equinox of J2000.

### 3.2 Moon and Earth-Moon secular polynomials (Table 1)

General formulation: `a(t) = a^{(0)} + a^{(1)} t + a^{(2)} t² + a^{(3)} t³ + a^{(4)} t⁴`,
`t` in TDB Julian centuries from J2000. Nominal values **before correction**:

| Argument | a^{(0)}                              | a^{(1)} (″/cy)             | a^{(2)} (″/cy²)  | a^{(3)} (″/cy³)     | a^{(4)} (″/cy⁴)     |
|----------|--------------------------------------|----------------------------|------------------|---------------------|---------------------|
| W₁       | 218° 18′ 59.95571″                   | 1 732 559 343.73604        | −6.8084          | +0.006604           | −0.00003169         |
| W₂       | 83° 21′ 11.67475″                    | 14 643 420.3171            | −38.2631         | −0.045047           | +0.00021301         |
| W₃       | 125° 02′ 40.39816″                   | −6 967 919.5383            | +6.3590          | +0.007625           | −0.00003586         |
| T        | 100° 27′ 59.13885″                   | 129 597 742.2930           | −0.0202          | +0.000009           | +0.00000015         |
| ϖ′       | 102° 56′ 14.45766″                   | 1 161.24342                | +0.529265        | −0.00011814         | +0.000011379        |

**Derived shorthand** used elsewhere in the note: `ν ≡ W₁^{(1)}`, `n′ ≡ T^{(1)}`.

> Note: the elpmpp02.pdf prints both the t³ and t⁴ rows for ϖ′ ending in
> "t³"; the trailing power is t⁴ on the last term, as is standard for the
> ELP polynomial form, and as implied by Table 6 listing only t² and t³
> corrections to W₂/W₃ but t³ and t⁴ corrections to W₁. The implementer
> should treat the last column as the t⁴ coefficient.

### 3.3 The argument ζ (precession-tied angle)

(elpmpp02.pdf §3.1, last paragraph.)

```
ζ = W₁ + (p + Δp) · t
```
with

```
p  = 5029.0966″/cy           (IAU 1976 precession constant)
Δp = −0.29965″/cy            (Herring et al. 2002 correction)
```

### 3.4 Planetary arguments (Table 2)

The planetary arguments `Me, V, Ma, J, S, U, N` are the **linear parts** of the
mean mean longitudes of Mercury, Venus, Mars, Jupiter, Saturn, Uranus, Neptune
from VSOP2000 (Moisson, 2000). Form: `λ = λ^{(0)} + λ^{(1)} t`.

| Planet      | λ^{(0)}                | λ^{(1)} (″/cy)        |
|-------------|------------------------|-----------------------|
| Mercury (Me)| 252° 15′ 03.216919″    | 538 101 628.68888     |
| Venus (V)   | 181° 58′ 44.758419″    | 210 664 136.45777     |
| Mars (Ma)   | 355° 26′ 03.642778″    |  68 905 077.65936     |
| Jupiter (J) |  34° 21′ 05.379392″    |  10 925 660.57335     |
| Saturn (S)  |  50° 04′ 38.902495″    |   4 399 609.33632     |
| Uranus (U)  | 314° 03′ 04.354234″    |   1 542 482.57845     |
| Neptune (N) | 304° 20′ 56.808371″    |     786 547.89700     |

(Note: The planetary argument named "T" in φ-of §2.2 is not a planet —
it is the heliocentric mean longitude of the Earth-Moon barycenter, the
**T** of §3.2 above. The label clash is intrinsic to the explanatory note.)

---

## 4. Constants and corrections

### 4.1 Fixed dimensionless constants (elpmpp02.pdf §4.3.1)

```
m = n′/ν     = 0.074801329
α = a₀/a′    = 0.002571881
```

`a₀`, `a′` are the keplerian semi-major axes of the Moon and the Earth-Moon
barycenter. The numerical values `a₀(DE405)` and `a₀(ELP)` used in §1.1 are:

```
a₀(DE405) = 384747.961370173 km
a₀(ELP)   = 384747.980674318 km
```

### 4.2 Fitted-correction tables — Table 3 (elpmpp02.pdf §4.2)

Notation: `ELP/MPP02(405)` = fit to JPL DE405 over [1950, 2060]; 
`ELP/MPP02(LLR)` = fit to LLR observations 1970–2001 (14500 normal points,
4 stations, 4 reflectors). All values in arcsec, except as noted.

| Correction              | ELP/MPP02(405) | ELP/MPP02(LLR) | Units      |
|-------------------------|----------------|----------------|------------|
| ΔW₁^{(0)}               | −0.07008       | −0.10525       | arcsec     |
| ΔW₂^{(0)}               | +0.20794       | +0.16826       | arcsec     |
| ΔW₃^{(0)}               | −0.07215       | −0.10760       | arcsec     |
| ΔW₁^{(1)} = Δν          | −0.35106       | −0.32311       | arcsec/cy  |
| ΔW₂^{(1)}               | +0.08017       | +0.08017       | arcsec/cy  |
| ΔW₃^{(1)}               | −0.04317       | −0.04317       | arcsec/cy  |
| ΔW₁^{(2)}               | −0.03743       | −0.03794       | arcsec/cy² |
| ΔΓ                      | +0.00085       | +0.00069       | arcsec     |
| ΔE                      | −0.00006       | +0.00005       | arcsec     |
| ΔT^{(0)}                | −0.00033       | −0.04012       | arcsec     |
| ΔT^{(1)} = Δn′          | +0.00732       | +0.01442       | arcsec/cy  |
| Δϖ′^{(0)}               | −0.00749       | −0.04854       | arcsec     |
| Δe′                     | +0.00224       | +0.00226       | arcsec     |

### 4.3 Additive secular corrections — Table 6 (elpmpp02.pdf §4.3.3)

These are applied **only** for the DE405 fit, on top of Table 3, to keep
ELP/MPP02(405) close to DE406 over 6 millennia.

| Coefficient        | Value                  |
|--------------------|------------------------|
| ΔW₁^{(3)}          | −0.00018865 ″/cy³      |
| ΔW₁^{(4)}          | −0.00001024 ″/cy⁴      |
| ΔW₂^{(2)}          | +0.00470602 ″/cy²      |
| ΔW₂^{(3)}          | −0.00025213 ″/cy³      |
| ΔW₃^{(2)}          | −0.00261070 ″/cy²      |
| ΔW₃^{(3)}          | −0.00010712 ″/cy³      |

### 4.4 Implementation of the corrections — eq. block of §4.3.1

Define the auxiliary corrections to (ν, Γ, E, e′, n′) (Table 4 of elpmpp02.pdf):

```
δν  = +0.55604″/cy   + ΔW₁^{(1)}
δΓ  = −0.08066″      + ΔΓ
δE  = +0.01789″      + ΔE
δe′ = −0.12879″      + Δe′
δn′ = −0.0642″/cy    + Δn′
```
(The leading numerical parts come from the previous fit of ELP to DE200; the
`Δ` corrections come from Table 3 above.)

#### 4.4.1 Correction to coefficients of the Main-Problem periodic series

For longitude (S1) and latitude (S2):
```
δA_{i} = −m B₁^{[i]} (1 + 2α/(3m) · B₅^{[i]}) · (δν / ν)
       +    B₁^{[i]} (1 + 2α/(3m) · B₅^{[i]}) · (δn′ / ν)
       + ( B₂^{[i]} δΓ + B₃^{[i]} δE + B₄^{[i]} δe′ )
```

For distance (S3):
```
δA_{i} = −m B₁^{[i]} (1 + 2α/(3m) · B₅^{[i]} + 2 A_{i} / (3m) ) · (δν / ν)
       +    B₁^{[i]} (1 + 2α/(3m) · B₅^{[i]} )                · (δn′ / ν)
       + ( B₂^{[i]} δΓ + B₃^{[i]} δE + B₄^{[i]} δe′ )
```
(The last bracket is to be expressed in radian for consistency with the
arcsec/km coefficient convention; see elpmpp02.pdf §4.3.1, last sentence.)

#### 4.4.2 Corrections to secular coefficients of W₁, W₂, W₃, T, ϖ′

Direct table-driven additions (elpmpp02.pdf §4.3.2):

```
W₁^{(0)}  ← W₁^{(0)}  + ΔW₁^{(0)}
W₂^{(0)}  ← W₂^{(0)}  + ΔW₂^{(0)}
W₃^{(0)}  ← W₃^{(0)}  + ΔW₃^{(0)}
T^{(0)}   ← T^{(0)}   + ΔT^{(0)}
ϖ′^{(0)}  ← ϖ′^{(0)}  + Δϖ′^{(0)}

W₁^{(1)}  ← W₁^{(1)}  + ΔW₁^{(1)}
T^{(1)}   ← T^{(1)}   + ΔT^{(1)}
W₁^{(2)}  ← W₁^{(2)}  + ΔW₁^{(2)}
W₂^{(1)}  ← W₂^{(1)}  + ΔW₂^{(1)} + δW₂^{(1)}
W₃^{(1)}  ← W₃^{(1)}  + ΔW₃^{(1)} + δW₃^{(1)}
```

The supplementary `δW₂^{(1)}` and `δW₃^{(1)}` come from the closed form of
elpmpp02.pdf §4.3.2:

```
δW_{i=2,3}^{(1)} = [ W_i^{(1)} / ν − m (B′_{i,1} + 2α/(3m) · B′_{i,5}) ] · ΔW₁^{(1)}
                 + ( B′_{i,1} + 2α/(3m) · B′_{i,5} )                    · ΔT^{(1)}
                 + ν · ( B′_{i,2} ΔΓ + B′_{i,3} ΔE + B′_{i,4} Δe′ )
```
with the `B′_{i,j}` of Table 5 (elpmpp02.pdf §4.3.2):

| j | constant σ′_j | B′_{2,j} = ∂W₂^{(1)} / (∂σ′_j · ν) | B′_{3,j} = ∂W₃^{(1)} / (∂σ′_j · ν) |
|---|---------------|-----------------------------------|-----------------------------------|
| 1 | ν             | +0.311079095                       | −0.103837907                      |
| 2 | Γ             | −0.004482398                       | +0.000668287                      |
| 3 | E             | −0.001102485                       | −0.001298072                      |
| 4 | e′            | +0.001056062                       | −0.000178028                      |
| 5 | n′            | +0.000050928                       | −0.000037342                      |

The last term in the δW formula must be expressed in radian.

#### 4.4.3 DE405-only additive corrections

For the DE405 fit only, additionally apply Table 6 (§4.3 above):
```
W₁^{(3)} ← W₁^{(3)} + ΔW₁^{(3)}
W₁^{(4)} ← W₁^{(4)} + ΔW₁^{(4)}
W₂^{(2)} ← W₂^{(2)} + ΔW₂^{(2)}
W₂^{(3)} ← W₂^{(3)} + ΔW₂^{(3)}
W₃^{(2)} ← W₃^{(2)} + ΔW₃^{(2)}
W₃^{(3)} ← W₃^{(3)} + ΔW₃^{(3)}
```
For LLR fit, leave W^{(2..4)} at the Table 1 nominal values (no Table 6
addition).

> **`icor` parity warning to the implementer.** The IMCCE explanatory note
> §6 says `icor=1 ⇒ LLR`, `icor=2 ⇒ DE405`. The reference Fortran source
> `ELPMPP02.for` (same distribution, header comments at lines 148–151 and
> 215–217 and the test driver lines 24, 53) uses `icor=0 ⇒ LLR`,
> `icor=1 ⇒ DE405`. Both conventions describe the **same physical fits**.
> The clean-room Rust API should pick its own enum (e.g. `Fit::Llr`,
> `Fit::De405`) to avoid the off-by-one trap and document which prose
> mapping it follows.

---

## 5. Path-taken note

**Path A — primary LLR-fit + DE405-fit table located in IMCCE primary.**

The constants needed for both supported fits (LLR and DE405) are explicitly
tabulated in IMCCE primary documents:

- elpmpp02.pdf Tables 3, 4, 5, 6 — the closed-form correction coefficients.
  URL: `ftp://cyrano-se.obspm.fr/pub/2_lunar_solutions/2_elpmpp02/elpmpp02.pdf`
  SHA256: `08b988dda14deb8850f82ea4077115a6d44251c325dd48de137b15bc5c0c2c93`
- ELPMPP02.for — same numerical constants embedded in the reference Fortran
  routine, available for byte-exact cross-checking.
  URL: `ftp://cyrano-se.obspm.fr/pub/2_lunar_solutions/2_elpmpp02/ELPMPP02.for`
  SHA256: `3a95c77de63dddc4d438765da3f91598ddd4f1ce3601683cb2c1af8e4acd838f`
- README.TXT — confirms publication context, authorship (Chapront, Chapront,
  Francou — SYRTE), and the canonical reference paper (A&A 404, 735, 2003).
  URL: `ftp://cyrano-se.obspm.fr/pub/2_lunar_solutions/2_elpmpp02/README.TXT`
  SHA256: `aee2edbd7cc679fd6f1e871fb017493f075a6befc7f720f4f6bb8ee2b56e7fd8`

There was therefore no need to fall back to Path B (deriving constants from
the 2003 A&A paper alone).

---

## 6. Fetch manifest

**Fetch host:** `cyrano-se.obspm.fr` (IMCCE / SYRTE FTP server).
**Distribution path:** `/pub/2_lunar_solutions/2_elpmpp02/`.
**Fetch timestamp (UTC):** 2026-05-09T17:44Z.
**Local destination:** `/workspace/vedaksha/scripts/data/elpmpp02/`.

| File          | Size (bytes) | SHA256                                                              | Primary URL                                                                         |
|---------------|-------------:|---------------------------------------------------------------------|-------------------------------------------------------------------------------------|
| README.TXT    |        4 445 | `aee2edbd7cc679fd6f1e871fb017493f075a6befc7f720f4f6bb8ee2b56e7fd8`  | `ftp://cyrano-se.obspm.fr/pub/2_lunar_solutions/2_elpmpp02/README.TXT`              |
| elpmpp02.pdf  |      215 008 | `08b988dda14deb8850f82ea4077115a6d44251c325dd48de137b15bc5c0c2c93`  | `ftp://cyrano-se.obspm.fr/pub/2_lunar_solutions/2_elpmpp02/elpmpp02.pdf`            |
| ELPMPP02.for  |       28 112 | `3a95c77de63dddc4d438765da3f91598ddd4f1ce3601683cb2c1af8e4acd838f`  | `ftp://cyrano-se.obspm.fr/pub/2_lunar_solutions/2_elpmpp02/ELPMPP02.for`            |
| ELP_MAIN.S1   |      103 360 | `3602147c43b77f86394c9034ea0e66807c6a674eeac87ada2a23aecd328706f1`  | `ftp://cyrano-se.obspm.fr/pub/2_lunar_solutions/2_elpmpp02/ELP_MAIN.S1`             |
| ELP_MAIN.S2   |       92 755 | `c06fca782f973a5365a4a19dd8b8a2a5ce711063e007ad929be6206686e459b8`  | `ftp://cyrano-se.obspm.fr/pub/2_lunar_solutions/2_elpmpp02/ELP_MAIN.S2`             |
| ELP_MAIN.S3   |       71 141 | `22f2cebde62d7451bc984ea67716b32091848c6f33ce746a9d1ba5de76074a56`  | `ftp://cyrano-se.obspm.fr/pub/2_lunar_solutions/2_elpmpp02/ELP_MAIN.S3`             |
| ELP_PERT.S1   |    1 209 918 | `222b2895f476370e93b05c50bc207d5f637ca3cd7002f848054ff44b9f1742ba`  | `ftp://cyrano-se.obspm.fr/pub/2_lunar_solutions/2_elpmpp02/ELP_PERT.S1`             |
| ELP_PERT.S2   |      668 038 | `0fd9af9d5e79fb9315c2ea295c8abe8f1ca385401fa93c8a032d64eb45c7d209`  | `ftp://cyrano-se.obspm.fr/pub/2_lunar_solutions/2_elpmpp02/ELP_PERT.S2`             |
| ELP_PERT.S3   |    1 281 928 | `15123e2eb0683ebffacc2b67339693532060a4db1f9c26eba502e0dad941d216`  | `ftp://cyrano-se.obspm.fr/pub/2_lunar_solutions/2_elpmpp02/ELP_PERT.S3`             |

The `ftp://ftp.imcce.fr/pub/ephem/moon/elpmpp02/` path suggested in the task
**does not exist**; that FTP host serves only ELP82B and a few other lunar
products under `/pub/ephem/moon/`. The actual canonical IMCCE distribution of
ELP/MPP02 lives on `cyrano-se.obspm.fr` as listed above. This was confirmed by
listing `/pub/ephem/moon/` (which contains only `elp82b/`, `geopos/`, `moonaps/`,
`moonphas/`, `rectcd/`, `riseset/`, plus a README) and then by directly
listing the cyrano-se path successfully.

---

## 7. Worked numerical examples (Tier-2 acceptance tests)

Source: elpmpp02.pdf §6, Tables 8.a (printed twice, "LLR fit" and
"DE405 fit + secular corrections"). All coordinates are
**geocentric, inertial mean ecliptic and equinox of J2000**, with positions in
km and velocities in km/day. Each `(JD, X, Y, Z, Ẋ, Ẏ, Ż)` row constitutes a
single acceptance vector.

### 7.1 ELP/MPP02(LLR) — fit to LLR observations

| Julian Date (TDB)       | Date (0h TDB)   | x_2000^E (km)        | y_2000^E (km)        | z_2000^E (km)       | ẋ_2000^E (km/day) | ẏ_2000^E (km/day) | ż_2000^E (km/day) |
|-------------------------|-----------------|----------------------|----------------------|---------------------|-------------------|-------------------|-------------------|
| JD 2 444 239.5          | 1980-Jan-01     |  43 890.28240        | 381 188.72745        | −31 633.38165       | −87 516.19748     |  13 707.66444     |   2 754.22124     |
| JD 2 446 239.5          | 1985-Jun-23     | −313 664.59645       | 212 007.26674        |  33 744.75120       | −47 315.91281     | −75 710.87501     |  −1 475.62869     |
| JD 2 448 239.5          | 1990-Dec-14     | −273 220.06067       | −296 859.76822       | −34 604.35700       |  60 542.32759     | −58 162.31674     |   2 270.88691     |
| JD 2 450 239.5          | 1996-Jun-05     |  171 613.14280       | −318 097.33750       |  31 293.54824       |  83 266.77990     |  42 585.83028     |  −1 695.82611     |
| JD 2 452 239.5          | 2001-Nov-26     |  396 530.00635       |  47 487.92249        | −36 085.30903       | −12 664.28694     |  83 512.75719     |   1 507.36756     |

### 7.2 ELP/MPP02(DE405) + Table-6 secular corrections — long-range check

| Julian Date (TDB)       | Date (0h TDB)   | x_2000^E (km)        | y_2000^E (km)        | z_2000^E (km)       | ẋ_2000^E (km/day) | ẏ_2000^E (km/day) | ż_2000^E (km/day) |
|-------------------------|-----------------|----------------------|----------------------|---------------------|-------------------|-------------------|-------------------|
| JD 2 500 000.5          | 2132-Sep-01     |  274 034.59103       |  252 067.53689       | −18 998.75519       | −62 463.61338     |  65 693.96392     |   6 595.32890     |
| JD 2 300 000.5          | 1585-Feb-01     |  353 104.31359       | −195 254.11808       |  34 943.54592       |  39 543.13678     |  74 373.18070     |   −700.65351      |
| JD 2 100 000.5          | 1037-Jun-28     |  −19 851.27674       | −385 646.17717       | −27 597.66134       |  87 539.40744     |  −7 599.68484     |  −4 960.44360     |
| JD 1 900 000.5          |  489-Dec-02     | −370 342.79254       |  −37 574.25533       |  −4 527.91840       |  12 255.28746     | −89 710.97508     |   7 649.44285     |
| JD 1 700 000.5          |  −58-May-08     | −164 673.04720       |  367 791.71329       |  31 603.98027       | −75 884.68815     | −35 802.26558     |  −4 239.59895     |

> **Tier-2 tolerance recommendation.** elpmpp02.pdf §8 ("comparing
> ELP/MPP02(LLR) to DE405/DE406") quantifies the inherent precision of the
> series: 0.06″ / 0.003″ / 4 m over [1950, 2060]; 0.6″ / 0.05″ / 50 m over
> [1500, 2500]; 50″ / 5″ / 10 km over [−3000, 3000]. The implementer should
> reproduce the printed digits in §7.1 and §7.2 to about ±5 in the last
> printed decimal (i.e. ±5×10⁻⁵ km in position and ±5×10⁻⁵ km/day in
> velocity) — this is a tighter check than the inherent precision because
> the printed values are the actual reference subroutine output, not
> independent observations.

### 7.3 Independent A&A 2003 numerical citation

Chapront & Francou (2003), abstract: ELP/MPP02(LLR) reproduces a
two-millennia LLR-anchored ephemeris with residuals "less than 0.03 arcsec
in longitude and latitude and 30 m in distance". This abstract figure is
informational only; binding numerical acceptance vectors are §7.1/§7.2.

---

## 8. Open questions and ambiguities

1. **`l′` argument convention.** elpmpp02.pdf §3.1 prints
   `l′ = T − ϖ′`. The argument identifier `l′` is the Sun's mean anomaly,
   conventionally `λ_Sun − ϖ_Sun`, equivalently `T − ϖ′` with the
   present labels (T = mean longitude of EMB ≈ Sun's apparent geocentric
   mean longitude − 180°, ϖ′ = perihelion of EMB = aphelion of Sun in
   geocentric reckoning). The implementer should verify against the FORTRAN
   reference that no extra ±180° offset is buried in the convention; the
   note prints the formula bare.

2. **Time-scale of reference values.** The Fortran subroutine accepts `tj`
   as days from J2000 with no explicit time-scale tag in its API. The
   explanatory note specifies TDB. The implementer should follow TDB and
   document the choice in the public Rust API.

3. **Velocity unit derivation.** The note prints the velocity check values
   in km/day (Table 8.a) but does not give the velocity series in closed
   form. §2.4 above provides the analytic derivative scheme; the
   implementer must verify against the printed velocities. Mind the
   `1 / 36525` factor when going from `t` (centuries) to time derivative
   per day, and the arcsec→radian conversion before P/Q rotation.

4. **`t³` vs `t⁴` printout in Table 1 for ϖ′.** elpmpp02.pdf Table 1 last
   row prints both the third and fourth coefficients with a `t³`
   superscript. The implementer should use the standard polynomial-degree
   convention (last entry = t⁴ coefficient) — see §3.2 note above.

5. **`icor` enum parity.** Spec §4.4.3 calls this out: the prose §6 says
   `icor=1 ⇒ LLR` while the supplied Fortran source uses
   `icor=0 ⇒ LLR`. The clean-room implementation should adopt a
   symbolic enum and document its mapping to both conventions.

6. **Earth's-figure / lunar-figure / relativistic / tidal terms.** The
   README.TXT enumerates these as components of ELP/MPP02. The
   explanatory note treats them as already incorporated into the perturbation
   files (`ELP_PERT.S{1,2,3}`); no further user-side action is required and
   the implementer should not attempt to add them separately.

---

## 9. Forbidden-source incidents

During source-discovery web searches for the IMCCE FTP location, hit lists
included two third-party implementations of ELP/MPP02 (`github.com/ytliu0/...`
and a SourceForge wiki). Per the SPEC subagent contract, **no content from
those pages was fetched**: the search snippet that surfaced the canonical IMCCE
FTP path (`ftp://cyrano-se.obspm.fr/pub/2_lunar_solutions/2_elpmpp02`) was the
single piece of information used from that search, and it was independently
verified by directly listing the FTP directory. No third-party implementation
was opened, read, paraphrased, or referenced for any formula, constant, or
test value in this spec.

---

*End of spec.*
