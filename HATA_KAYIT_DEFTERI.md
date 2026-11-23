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
