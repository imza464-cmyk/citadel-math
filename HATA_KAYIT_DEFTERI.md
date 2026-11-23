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
