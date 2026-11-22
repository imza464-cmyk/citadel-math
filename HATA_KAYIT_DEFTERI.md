# 🛡️ MATEMATİK ÇEKİRDEĞİ (`kmath`) HATA KAYIT DEFTERİ (BUG LOGBOOK & ANTI-REGRESSION SHIELD)

Bu belge, **Matematik Çekirdeği (`kmath` / `Citadel-Math`)** projesinde tespit edilen sembolik doğrulama, aralık aritmetiği, boyut analizi ve sistem hatalarını kronolojik sırayla tutar.

---

## 📋 Çiğnenemez Altın Kurallar Matrisi (`kmath`)

1. **Boyutsal Tutarlılık Değişmezi:** Boyutları uyuşmayan iki fiziksel büyüklük toplanamaz, çıkarılamaz veya eşitlenemez (`kmath_dimension_check`).
2. **Host PC Sanal Bellek Koruması:** Python 3.14 üzerinde `import numpy` / `import scipy` çağrılarından önce `os.environ.setdefault("OPENBLAS_NUM_THREADS", "1")` tanımlanması ZORUNLUDUR (süreç başına `640 MB` sanal bellek şişmesini engeller).
3. **Kayıt Disiplini:** Tespit edilen ve çözülen her kritik matematiksel veya sayısal hata bu deftere ampirik doğrulama sonucuyla işlenir.
