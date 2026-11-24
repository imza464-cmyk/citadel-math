# 🛡️ MATEMATİK ÇEKİRDEĞİ (`kmath`) HATA KAYIT DEFTERİ (BUG LOGBOOK & ANTI-REGRESSION SHIELD)

Bu belge, **Matematik Çekirdeği (`kmath` / `Citadel-Math`)** projesinde tespit edilen sembolik doğrulama, aralık aritmetiği, boyut analizi ve sistem hatalarını kronolojik sırayla tutar.

---

## 📋 Çiğnenemez Altın Kurallar Matrisi (`kmath`)

1. **Boyutsal Tutarlılık Değişmezi:** Boyutları uyuşmayan iki fiziksel büyüklük toplanamaz, çıkarılamaz veya eşitlenemez (`kmath_dimension_check`).
2. **Host PC Sanal Bellek Koruması:** Python 3.14 üzerinde `import numpy` / `import scipy` çağrılarından önce `os.environ.setdefault("OPENBLAS_NUM_THREADS", "1")` tanımlanması ZORUNLUDUR (süreç başına `640 MB` sanal bellek şişmesini engeller).
3. **Kayıt Disiplini:** Tespit edilen ve çözülen her kritik matematiksel veya sayısal hata bu deftere ampirik doğrulama sonucuyla işlenir.


---

## 🔒 2026-11-23 — `sympy.sympify()` Üzerinden Prompt-Injection / Keyfi Kod Çalıştırma (RCE) Koruması

- **Belirti:** `kmath_mcp.py`, `engine/dimensions.py`, `engine/inventor.py` ve `kmath.py` içinde matematiksel ifadeler doğrudan `sp.sympify(..., evaluate=False)` ile ayrıştırılıyordu.
- **Kök Neden (Adli Kanıt):** SymPy'nin `sympify()` fonksiyonu arka planda Python `eval()` çalıştırdığı için, bir MCP istemcisinden gelen kötü niyetli string (`__import__('os').system(...)`, `().__class__.__bases__`, `lambda`) keyfi kod çalıştırma (RCE) riski oluşturuyordu.
- **Çözüm:** `engine/dimensions.py` içine Python'un kod yürütmeyen `ast.parse(..., mode="eval")` ayrıştırıcısına dayalı, yalnızca beyaz-listeli aritmetik işleçleri ve 15 temel matematik fonksiyonunu kabul eden `safe_parse_math_expr()` yazıldı; tüm `sp.sympify()` çağrıları kaldırıldı (repo genelinde `0` `sympify()` / `eval()`).
- **Ampirik Doğrulama:** `tests/test_mcp_security.py` içinde 11 adet RCE/injection vektörünün reddedildiği ve 15 araçlı MCP köprüsünün çalıştığı doğrulandı; GitHub Actions Linux + Windows matrisine eklendi.


---

## 🛠️ 2026-11-23 — `mcp 2.x` Kırıcı Değişikliği (`FastMCP` -> `MCPServer`) ve `requirements.txt` Sürüm Sabitlemesi

- **Belirti:** Temiz Ubuntu CI ortamında `pip install -r requirements.txt` sonrası `import kmath_mcp` çağrısı `ModuleNotFoundError: No module named 'mcp.server.fastmcp'` ile çöktü.
- **Kök Neden:** `requirements.txt` dosyasında üst sürüm sınırı olmadan `mcp>=1.0.0` yazılıydı; `pip`, `FastMCP`'yi `MCPServer`'a taşıyan `mcp 2.x` sürümünü kurdu.
- **Çözüm:** `requirements.txt` majör sürüm sınırlarıyla (`sympy>=1.12,<2.0`, `mcp>=1.0.0,<2.0.0`, `networkx>=3.0,<4.0`) sabitlendi ve `kmath_mcp.py` içinde hem `mcp 1.x` (`FastMCP`) hem `mcp 2.x` (`MCPServer`) uyumlu geri-düşüm importu eklendi.
- **Ampirik Doğrulama:** `tests/test_mcp_security.py` yerel ortamda ve GitHub Actions `ubuntu-latest` + `windows-latest` matrisinde doğrulandı.


---

## 🛠️ 2026-11-23 — Windows `shutil.which()` Göreli Yol (`.\citadel_math_core-windows-x86_64.exe`) Normalizasyonu

- **Belirti:** `v0.2.0` GitHub Release ikilisi ve wheel paketi sıfırdan izole bir kullanıcı klasöründe kurularak test edildiğinde, `PATH` otomatik keşfinde `resolve_rust_bin()` Windows üzerinde mutlak yol yerine `.\citadel_math_core-windows-x86_64.exe` göreli yolunu döndürdü.
- **Kök Neden (Adli Kanıt):** Windows üzerinde `shutil.which()` geçerli çalışma dizinini (`.`) önceleyip göreli yol döndürür. `_run_rust_cli()` alt süreci `cwd=ROOT_DIR` (`site-packages`) ile başlattığından veya süreç sonradan `os.chdir()` yaptığında göreli yol taşınabilirliğini yitirir.
- **Çözüm:** Hem `kmath_mcp.py` hem de `kmath.py` içindeki `resolve_rust_bin()` fonksiyonunda `shutil.which()` çıktısı `os.path.abspath(found)` ile kesin mutlak yola dönüştürüldü; `tests/test_mcp_security.py` içine `test_resolve_rust_bin_path_normalizes_relative_which_result` regresyon testi eklendi.
- **Ampirik Doğrulama:** `python -m unittest discover -s tests -v` (`10/10 PASS`) ve canlı `v0.2.0` indirme/kurulum testinde 26/26 kapı PASS doğrulandı.


---

## 📱 2026-11-23 — Mobil (`390px`) Yatay Taşma (CSS Grid `min-width: 0` Eksikliği & Kırılmayan Monospace Dize)

- **Belirti:** 2026 yeniden tasarımında masaüstü (`1440px`) kusursuz görünürken gerçek telefon ekranında (`390px`) tüm sayfa sağ kenardan taşıp kesiliyordu ve üst çubuk 3 satıra yığılarak ekranın üstünü kaplıyordu.
- **Kök Neden (Adli Kanıt):** `.hero-split > *` ve kart kapsayıcılarında `min-width: 0` yoktu; `.console-inputs` (`1fr auto 1fr`) içindeki `<input>` elemanlarının tarayıcı doğal genişliği ve `DIMENSIONALLY_HOMOGENEOUS_BUT_UNIT_SCALE_MISMATCH` (49 karakter kırılmayan dize) `overflow-wrap: anywhere` olmadan `<main>` sütununu `390px` dışına genişletiyordu.
- **Çözüm:** Tüm grid/kart çocuklarına `min-width: 0; max-width: 100%`, uzun karar/kod dizelerine `overflow-wrap: anywhere` eklendi; `<= 640px` kırılımında üst çubuk kompakt 2 satıra indirildi (arama ikonu + ortalanan yatay kaydırmalı sekme şeridi `scrollIntoView({ inline: "center" })`), konsol girdileri dikey yığıldı ve gecikme çubuğu 2 sütunlu mobil düzene geçirildi.
- **Ampirik Doğrulama:** `390×844` gerçek iframe telemetrisinde **8/8 bölümde `overflow: false` (`scrollWidth <= innerWidth`)** ve 3 telefon ekranlı görsel piksel denetimi PASS.

---

## 🚀 2026-11-23 — `kmath` MCP 6-Maddelik Adli Denetim Sertleştirmesi ve Hohmann Enöteye Süzülme (Coast-to-Apogee) Güdümü

- **Belirti:** Canlı 15/15 MCP araç stres testinde 6 kritik hata yakalandı: (1) `kmath_solve_for` bilinmeyen sembolde `inferred_dimension: null` iken Rust docstring'ine sahte `[1 (Boyutsuz)]` ispat damgası basıyordu ve `±sqrt` dal uyarısı yoktu; (2) `kmath_stress_check` aynı girdide Python ve Rust arasında `102 MPa` Von Mises ve `104 °C` duvar sıcaklığı çelişkisi üretiyordu; (3) `kmath_physical_bounds_check` çoklu ihlalde (`v > c` ve `eta > Carnot`) ilk hatada kısa devre yapıp ikincisini yutuyordu; (4) `kmath_simulate_trajectory` `30.87 G` kısma-sız kimyasal roket, yörünge hızı altında `orbit_achieved: true` ve `kmath_inverse_design(1000kg, 400km, 4.5G)` için `0/6912` çözüm üretiyordu; (5) 15 FastMCP aracının JSON parametre açıklamaları boştu, `kmath_materials` geçersiz kategoride ipucu vermiyordu, `kmath_dimension_check` SI eksenlerini (`L`, `M L T^-2`) reddediyordu ve paralel MCP çağrıları senkron bloklanıyordu; (6) `kmath_synthesize("F,m,t", "E_k")` hipergraf kinematik köprü eksikliğinden `0` dönüyordu.
- **Kök Neden (Adli Kanıt):**
  - `codegen.rs` `Option::None` boyutu `Dimension::dimensionless()` gibi biçimlendiriyordu.
  - `kmath_stress_check` hem eski Python kopya formüllerini hem Rust motorunu birlikte çağırarak SSOT'u bozuyordu.
  - `simulator.rs::guidance_pitch_angle` içinde `vh_des = (1.65 * dh / t_go).max(vh_phase)` ifadesi `vx -> v_circ` yaklaştıkça `t_go -> 6s` çökmesiyle hedef yörüngenin hemen altında `+2200 m/s` dikey tırmanış emrediyor, `Beamed SSTO` ve `400 km` adaylarını `e = 0.27..2.35` hiperbolik kaçışa fırlatarak `SECO (e < 0.08)` koşulunun hiç tetiklenmemesine yol açıyordu; `engine/simulator.py` ise açık döngü `_pitch_angle(alt)` kullandığından `14.986 m/s` kaçışa sapıyordu.
- **Çözüm:**
  - `codegen.rs` / `main.rs`: `dimension_verified` bayrağı, `[Bilinmiyor / Doğrulanmadı — SSOT Ontoloji Dışı Sembol]` dürüst etiketi ve `±sqrt` dal uyarısı eklendi.
  - `kmath_mcp.py` / `materials.rs`: `kmath_stress_check` tekil Rust SSOT'a bağlandı (`tank_pressure_bar` aktif, `bare_overall_pass` vs `tps_protected_overall_pass` ayrıldı); `physical_bounds.rs` tüm eşzamanlı ihlalleri `violations[]` listesinde topluyor; 15 FastMCP aracına `Annotated[..., Field(description=...)]`, asenkron thread-pool (`_async_mcp_tool`), SI eksen ayrıştırıcısı ve kategori ipuçları eklendi.
  - `simulator.rs` & `engine/simulator.py`: Hohmann enöteye süzülme (`t_apo > 0.55 * t_circ + 4.0` iken motor bekleme) + enöte daireselleştirme (`vh -> 0`) + frenleme zarfı (`vh_brake.min(vh_cap)`) + `max_g_throttle` (4.5 G) uygulandı; `designer.rs` yük-ölçekli ızgara ile güncellendi; `canonical_laws.json` ve `ONTOLOGY.json`'a 3 kinematik/iş yasası (`217` yasa / `356` sembol) eklenip `site/data/ssot_bundle.js` senkronlandı.
- **Ampirik Doğrulama:** `cargo clippy --all-targets -- -D warnings` (0 uyarı), `cargo test` (`34/34 PASS`, `test_hohmann_circular_orbit_400km_45g_and_beamed_200km` dahil), `kmath.py audit` (0 hata), `kmath.py rust audit` (`16/16 PASS`, `13.02 ms`), `unittest` (`10/10 PASS`), `verify_site.py` (`77 ID PASS`), sıkılaştırılmış `verify_6_fixes.py` (`21/21 PASS`: `chemical` `201.7 km, e=0.0003, 4.5 G`; `beamed` `204.7 km, e=0.0007, 4.5 G`; `inverse_design(1000kg, 400km, 4.5G)` `765` uygun / `6` Pareto `min_perigee=402.6 km`; 9-lu paralel FastMCP `0.028s`; `F,m,t -> E_k` & `W` sentezlendi).

---

## 🛡️ 2026-11-24 — `law_fanno_friction_parameter` 4× Darcy/Fanning Katsayı Hatası, Planck Ölçeği `1e-30` Payda Maskelemesi ve %100 (`307/307`) Altın Çapa Kilidi

- **Belirti:** 4-katmanlı epistemik öz-doğrulama kalkanının altın sayısal referans çapaları `45/307` (%14.7) kapsamından **`307/307` (%100.0)** zorunlu kapsama genişletilirken iki gerçek epistemik hata tespit edildi:
  1. `database/canonical_laws.json` içindeki `law_fanno_friction_parameter` yasası, `f_darcy` (Darcy sürtünme katsayısı) sembolünü kullanmasına rağmen `(4 * f_darcy * L) / D_pipe` hesaplayarak gerçek Fanno sonik tıkanma parametresini **4 kat (%400) fazla** üretiyordu — ancak `4` boyutsuz olduğu için $\mathbb{Z}^7$ SI boyut kalkanından sorunsuz geçmişti.
  2. `epistemic_audit.rs` (Kapı 17/18) ve `kmath.py` (Kapı 11) bağıl hata paydasında `max(|expected|, 1e-30)` tabanı kullanıyordu; Planck uzunluğu (`1.616e-35 m`) ve Planck zamanı (`5.391e-44 s`) gibi $10^{-30}$ altı beklenen değerlerde bu taban katsayı mutasyonlarını maskeleyebiliyordu.
- **Kök Neden (Adli Kanıt):**
  1. Akışkanlar mekaniğinde Fanning sürtünme katsayısı $f_F = f_D / 4$ olduğundan $L^* = 4 f_F L / D = f_D L / D$'dir. `law_darcy_weisbach_pressure_drop` yasası `f_darcy`'yi Darcy ($f_D$) olarak kullandığı halde `law_fanno_friction_parameter` Fanning'in `4` çarpanını `f_darcy` üzerine ikinci kez eklemişti.
  2. `1e-30` sıfıra bölme koruması kuantum/Planck ölçeğindeki fiziksel sabitlerden ($10^{-35}..10^{-44}$) 5–14 mertebe büyüktü.
- **Çözüm & Kalıcı Kural:**
  1. `law_fanno_friction_parameter` denklemi `(f_darcy * L) / D_pipe` olarak düzeltildi (Anderson, Shapiro, White 3-kaynak doğrulamalı) ve hem Rust Kapı 18 hem Python Kapı 12'ye **15. Çapraz-Yasa Kapanışı (`fanno_param == delta_P_dw / q_dyn`)** eklendi.
  2. Tüm **307/307 (%100.0)** kanonik yasaya sıfır-dışı bağımsız altın sayısal referans çapası (`reference_checks`) işlendi; Rust Kapı 17 ve Python Kapı 11 eksik çapa bırakılmasını **derleme/denetim hatası** yapacak şekilde katılaştırıldı.
  3. Bağıl hata payda tabanı her iki motorda `1e-30 -> 1e-300` (`> f64::MIN_POSITIVE`) yapıldı.
- **Ampirik Doğrulama:** `cargo clippy --all-targets -- -D warnings` (0 uyarı), `cargo test` (`37/37 PASS`), `cargo build --release` PASS, `kmath.py audit` (`13/13 PASS`, `307/307` çapa), `kmath.py rust audit` (`20/20 PASS`, `20.31 ms`), `unittest` (`10/10 PASS`), `verify_site.py` (`79 ID PASS`), `node --check site/data/ssot_bundle.js` PASS ve `mutation_probe.py` (`307/307 = %100` yasa çapası, `60/60 = %100` Ashby malzeme zarfı, 5/5 boyutsuz katsayı/işaret mutasyonu `25%–1155%` sapmayla YAKALANDI).

---

## 🛡️ 2026-11-24 — Evrensel `Dual64` Jakobyen Kalkanı ile 2 Durağan-Nokta Çapa Kör Noktasının Yakalanması & Faz 2 Genişletmesi (`367` Yasa / `618` Sembol / `80` Malzeme / `7/7` SI Ekseni)

- **Belirti:** Tüm yasalar için otomatik `Dual64` kısmi türev sıfır-dışılık taraması (`df/dx_i != 0`) devreye alındığında, altın çapası mevcut olan iki yasada girdi değişkenlerinden birinin kısmi türevinin tam **`0.0`** çıktığı tespit edildi:
  1. `law_specific_angular_momentum_orbit`: Dairesel yörünge (`e_orb = 0.0`) noktasında test edildiği için $\partial h/\partial e = -\mu a e / h = 0$ oluyordu; bu nedenle çapa `1 - e^2` yerine `1 + e^2` veya `1 - 2*e^2` mutasyonlarına tamamen kör kalıyordu.
  2. `law_von_mises_plane_stress`: $\sigma_{ax} = 100\text{ MPa}, \sigma_{hoop} = 200\text{ MPa}$ ($\sigma_{ax} = \sigma_{hoop}/2$) tam durağan noktasında test edildiği için $\partial\sigma_{vm}/\partial\sigma_{ax} = (2\sigma_{ax} - \sigma_{hoop})/(2\sigma_{vm}) = 0$ oluyordu.
- **Kök Neden:** Çapa test noktaları fiziksel simetri/durağan noktalara seçildiğinde bir girdi değişkeninin türevi sıfırlanarak o terimdeki katsayı/işaret hatası maskelenebilir.
- **Çözüm & Kalıcı Kural:**
  1. `law_specific_angular_momentum_orbit` çapası eliptik `e_orb = 0.6` (`sqrt(1-0.36)=0.8`), `law_von_mises_plane_stress` çapası `80 MPa / 200 MPa` durağan-dışı noktaya taşındı.
  2. Rust Kapı 17 (`epistemic_audit.rs`) içine **Evrensel `Dual64` Jakobyen Sıfır-Dışılık Kalkanı (`1.031/1.031` kısmi türev)** ve **%100 `>=3` Bağımsız Kaynak Zorunluluğu (`367/367` yasa)** eklendi.
  3. SSOT veritabanı **`307 -> 367` yasa** (`+60`: Nükleer/Füzyon, Elektrokimya/Batarya/Termoelektrik, Yarı İletken/PV, Kırılma/Triboloji, Hidroloji/Jeoteknik, Optik/Fotometri **7. SI ekseni `J` (kandela)**, Kimyasal Kinetik, Devre/Kontrol, Astrofizik), **`488 -> 618` tekil sembol** (`7/7` SI ekseni aktif: `M=227, L=376, T=308, I=50, Θ=31, N=14, J=4`), **`60 -> 80` malzeme** (`17` Ashby kategorisi: `+civil_structural`, `+elastomer_seal`, `+electrical_magnetic`, `+biomedical_alloy`), **`15 -> 25` çapraz-yasa kapanışı** ve **`17 -> 28` limit/monotonluk** denetimine yükseltildi.
- **Ampirik Doğrulama:** `cargo clippy --all-targets -- -D warnings` (0 uyarı, `1.12 s`), `cargo test` (`37/37 PASS`, `12.80 s`, tüm 367 yasada sembolik türev + çift-yönlü cebirsel tersleme dahil), `cargo build --release` PASS, `kmath.py audit` (`13/13 PASS`), `kmath.py rust audit` (`20/20 PASS`, `23.02 ms`), `unittest` (`10/10 PASS`), `verify_site.py` (`79 ID PASS`), `node --check site/data/ssot_bundle.js` (`588.6 KB`) PASS ve `mutation_probe.py` (`367/367 = %100` yasa, `80/80 = %100` malzeme, `5/5` mutasyon YAKALANDI).

---

## 🔬 2026-11-24 — Bağımsız Yüksek-Hassasiyetli Kahin (Oracle) ve 85-Testlik Modül Sertleştirmesi ile Yakalanan 5 Gerçek Sayısal Hata (`interval.rs`, `astrodynamics.rs`, `serde_json`)

- **Belirti:** Dış teknik inceleme önerisi doğrultusunda `interval.rs` için **25.000+ vektörlü sonsuz-hassasiyetli ikili rasyonel (`BigInt`) kahin**, `tests/test_interval_oracle.py` için **80-basamaklı bağımsız `decimal.Decimal` kahini** ve 0-testli `10` çekirdek modüle (`interval`, `dimensions`, `ast`, `buckingham`, `pareto`, `astrodynamics`, `materials`, `physical_bounds`, `codegen`, `graph`, `constants`) bağımsız ders kitabı / analitik birim testleri (`37 -> 85` Rust testi, `16/18` modül) yazıldığında **5 gerçek sayısal hata** yakalandı:
  1. **`interval.rs::pow` & `Dual64::pow` `< 1e-12` üs yapıştırma hatası:** `1000.0^(2.0 + 5e-13)` ifadesi `|(2+5e-13)-2| < 1e-12` nedeniyle sessizce tam sayı `powi(2)` yoluna saparak `[1e6, 1e6]` döndürüyor ve gerçek değerden ~30.000 ULP saparak IEEE 1788-2015 kapsama değişmezini kırıyordu.
  2. **`interval.rs::add_directed` büyük + subnormal uç hatası:** `f64::MAX + (-1.0)` için `a.abs() < 1e307` yedek yolu sahte `+Inf` üretiyordu; `0.5` ölçekli TwoSum ise `5e-324 + 1e307` birleşiminde subnormali `0.0`'a düşürüp `[1e307, 1e307]` (kapsama ihlali!) döndürüyordu.
  3. **`interval.rs::from_rel_uncertainty` & `from_pm` subnormal alt-taşması:** `center.abs() * rel_fraction` normal çarpımla yapıldığından `f64::MIN_POSITIVE * 1e-10` yarıçapı `0.0`'a çöküp belirsizliği yutuyordu.
  4. **`astrodynamics.rs` Stumpff `1e-5` yıkıcı sadeleşme (catastrophic cancellation):** `stumpff_c/s/c_prime/s_prime` fonksiyonlarındaki `1e-5` Taylor kesme eşiği, yakın-parabolik yörüngelerde (`1e-5 < |z| < 0.05`) `1 - cos(sqrt(z))` ve `1 - z*S - 2*C` çıkarmalarında **9–12 basamak** hassasiyet kaybettiriyordu.
  5. **`Cargo.toml` `serde_json` `float_roundtrip` eksikliği:** `serde_json = "1.0"` varsayılan hızlı ayrıştırıcısı MCP JSON sınırında `13.769785044591707` sayısını sessizce `+1 ULP` kaydırarak `13.769785044591709` okuyor; `exp(x)` bu 1-ULP girdi kaymasını `15 ULP` dışa-yuvarlama kapsama ihlaline büyütüyordu.
- **Çözüm & Kalıcı Kural:**
  1. `interval.rs`: `pow` ve `Dual64::pow` tam IEEE-754 kontrolüne (`p.fract() == 0.0`, `p == 0.5`) sabitlendi; `add_directed` 3-rejimli tam ispatlı yapıya (`|a|,|b|<1e307` TwoSum, `|a|,|b|>=1e290` `0.5`-ölçekli TwoSum, `big + small` doğrudan işaret-yönlü 1-ULP) geçirildi; `from_rel_uncertainty` üstten yuvarlamalı `mul_directed(...).1` + Knuth `add_directed` kullanıyor.
  2. `astrodynamics.rs`: Dört Stumpff fonksiyonunun Taylor eşiği 7-terimli tam seriyle (`kesme hatası < 5e-21`) **`0.1`**'e çıkarıldı.
  3. `citadel_math_core/Cargo.toml`: `serde_json = { version = "1.0", features = ["float_roundtrip"] }` zorunlu kılındı.
- **Ampirik Doğrulama:** `cargo clippy --all-targets -- -D warnings` (0 uyarı, `4.93 s`), `cargo test` (**`85/85 PASS`**, `16/18` modül, `25.000+` sonsuz-hassasiyetli ikili rasyonel kahin vektörü + Curtis Örnek 5.2 Lambert + SMAD SSO + Taylor-Sedov + analitik 2B Pareto HV), `cargo build --release` PASS, `kmath.py audit` (`13/13 PASS`), `kmath.py rust audit` (`20/20 PASS`, `22.88 ms`), `python -m unittest discover -s tests -v` (**`13/13 PASS`**, canlı Rust ikilisi üzerinde 80-basamaklı bağımsız `Decimal` kahini `tests/test_interval_oracle.py` dahil).
