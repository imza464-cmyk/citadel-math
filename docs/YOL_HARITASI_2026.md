# Copyright (c) 2026 Mekserina Karaosmanoğlu. Source-Available (Read & Run Only) — All Modification & Derivative Rights Reserved.
# MATEMATİK ÇEKİRDEĞİ (`citadel_math_core`) — STRATEJİK MÜHENDİSLİK YOL HARİTASI

> **Durum Notu (Tarihsel Arşiv):** Bu belge projenin ilk Faz 0 (20 yasa) aşamasında hazırlanmış tasarım yol haritasıdır. Aşağıda planlanan Faz 1–5 ve Faz 6 (214 kanonik yasa, 353 SSOT sembolü, 30 malzeme, 15 araçlı `kmath_mcp.py` ve saf-Rust `citadel_math_core`) mevcut `main` dalında uygulanmıştır; güncel mimari için bkz. [`README.md`](../README.md).
>
> **Vizyon:** Yapay zekanın ve insan mühendislerin fiziksel sistem (havacılık, uzay, itki, elektromanyetik, enerji ve yapısal mekanik) tasarımlarında **sıfır boyutsal hata, sıfır tanım kümesi ihlali ve sıfır malzeme/termal halüsinasyon** ile saniyeler içinde doğrulanmış ön tasarım (preliminary design) ve denklem sentezi yapmasını sağlayan **Deterministik Ön-Tasarım ve Boyutsal Doğrulama Motoru**.

---

## 📍 Mevcut Durum (Faz 0 — Tamamlandı ve Mühürlendi)

1. **$\mathbb{Z}^7$ Sembolik AST Boyut Kalkanı:** [`dimensions.py`](../engine/dimensions.py) ve [`dimensions.rs`](../citadel_math_core/src/dimensions.rs) üzerinde her denklemin sağ ve sol tarafı (`dim(LHS) == dim(RHS)`) toplama, çarpma, rasyonel üs ve aşkın fonksiyon kurallarıyla denetleniyor (20/20 yasa PASS).
2. **Küçük Doğrulama Çekirdeği ($\mathbb{Z}[x]$ & Yan Koşul Korunumu):** [`proof_kernel.rs`](../citadel_math_core/src/proof_kernel.rs) üzerinde taşma korumalı (`checked_i64`) kanonik polinom halkası (`PolyZ`), $(x^2-1)/(x-1) = x+1 \ [x \neq 1]$ yan koşul bekçisi (`GuardedPoly`) ve bağımsız kanıt denetleyicisi (`ProofVerifier`) aktif.
3. **Birleşik (Conjunctive) AND-OR Hipergraf Sentezleyici:** [`knowledge_graph.py`](../engine/knowledge_graph.py) ve [`graph.rs`](../citadel_math_core/src/graph.rs) üzerinde çok girdili denklemleri topolojik sıra ile birleştirip yeni formüller türeten `solve_conjunctive` motoru çalışıyor (5.6 µs).
4. **Gerçek 4-Aşamalı Runge-Kutta ($\mathcal{O}(\Delta t^4)$) & İnce Cidar/Termal Kalkan:** [`simulator.rs`](../citadel_math_core/src/simulator.rs) ve [`materials.rs`](../citadel_math_core/src/materials.rs) üzerinde 2D yörünge entegratörü, iki eksenli Von Mises kabuk gerilmesi ve Sutton-Graves radyatif ısınma dengesi mikrosaniyeler içinde çalışıyor.

---

## 🧭 5 Fazlı Mühendislik Yol Haritası

```
[FAZ 1: Tekil Gerçeklik & Birim Kalkanı]
        │
        ▼
[FAZ 2: Kapalı Döngü Tersine Tasarım (Inverse Design & Pareto Optimizer)]
        │
        ▼
[FAZ 3: Çok Değişkenli Kesirli Cebir Q(x1..xn) & Aralık Aritmetiği]
        │
        ▼
[FAZ 4: Dikey Mühendislik Kütüphaneleri (150+ Kanonik Yasa & 50+ Malzeme)]
        │
        ▼
[FAZ 5: Otonom Mühendis Ajan Entegrasyonu & MCP Sunucusu (JSON-RPC / kmath-mcp)]
```

---

### FAZ 1: Tekil Gerçeklik Kaynağı (SSOT) ve Ölçekli SI Birim Kalkanı (Hedef: 1–2 Hafta)
**Sorun:** Şu an Python tarafı [`canonical_laws.json`](../database/canonical_laws.json) dosyasından 20 yasayı okurken, Rust çekirdeği ([`graph.rs`](../citadel_math_core/src/graph.rs)) kendi içindeki 8 yasayı statik yüklüyor. Ayrıca boyut homojenliği ($\text{L}^1\text{T}^{-1}$) `km/s` ile `m/s` arasındaki $10^3$ ölçek farkını tek başına yakalamaz.
**Yapılacaklar:**
1. **JSON $\rightarrow$ Rust Doğrudan Derleme / Yükleme:** `citadel_math_core` doğrudan `database/canonical_laws.json` ve `database/materials.json` dosyalarını okuyacak ve sembolik ifade ayrıştırıcısı (`Expr::parse_str`) ile tüm 20+ yasayı tek kaynaktan (SSOT) çalıştıracak.
2. **Ölçekli Birim Çevirici (`ScaledUnit`):** Her değişken yalnızca $\mathbb{Z}^7$ boyut vektörünü değil, kanonik SI çarpanını ($\text{kPa} \rightarrow 10^3\text{ Pa}$, $\text{mm} \rightarrow 10^{-3}\text{ m}$, $\text{km} \rightarrow 10^3\text{ m}$) taşıyacak; birim dönüşüm hataları derleme/denetim anında sıfırlanacak.
3. **Fiziksel Geçerlilik Aralıkları (`DomainBounds`):** Her kanonik yasaya geçerlilik bölgesi eklenecek (ör. $\text{Mach} < 0.3$ sıkıştırılamaz Bernoulli vs. $\text{Mach} > 5$ hipersonik Sutton-Graves; $v \ll c$ klasik mekanik vs. $v \to c$ Lorentz). Geçerlilik bölgesi dışındaki denklem zincirlemeleri reddedilecek.

---

### FAZ 2: Kapalı Döngü "Tersine Mühendislik ve Tasarım Optimizatörü" (Inverse Design Engine) (Hedef: 2–4 Hafta)
**Sorun:** Şu an `invent`, `simulate` ve `stress` birbirinden ayrı üç komut olarak çalışıyor. Gerçek dünyada mühendis formül aramaz; **kısıtları sağlayan çalışan tasarım** arar.
**Yapılacaklar:**
1. **Kapalı Döngü Sentez Boru Hattı (`Synthesize ➔ Simulate ➔ Stress-Test ➔ Filter`):**
   * Kullanıcı hedefi verir:
     ```bash
     kmath design --payload 500kg --orbit 400km --max-g 6.0 --safety-factor 1.5
     ```
   * **Adım 1 (Hipergraf):** İtki ve kütle denklemlerini sentezler.
   * **Adım 2 (Rust RK4 Grid/Nelder-Mead/Sobol Taraması):** Rust çekirdeği saniyede **2.000+ yörünge simülasyonu** koşturarak yakıt kütlesi, yanma süresi, gövde yarıçapı ve et kalınlığı uzayını tarar.
   * **Adım 3 (Malzeme & Termal Elek):** `materials.json` içindeki tüm malzemeleri (Al-Li 2195, T1000G, Inconel 718, SiC, Ti-6Al-4V) Von Mises akma, Euler burkulma (buckling) ve Sutton-Graves burun sıcaklığı testinden geçirir.
   * **Çıktı:** Fiziksel olarak eriyen veya kırılan tüm tasarımları eler; mühendise **Pareto-optimal en hafif / en ucuz 3 somut araç konfigürasyonunu** tablo ve kanıt iziyle verir.

---

### FAZ 3: Çok Değişkenli Rasyonel Fonksiyon Gövdesi $\mathbb{Q}(x_1, \dots, x_n)$ ve Aralık Aritmetiği (Hedef: 4–6 Hafta)
**Sorun:** [`proof_kernel.rs`](../citadel_math_core/src/proof_kernel.rs) şu an tek değişkenli $\mathbb{Z}[x]$ polinomlarını kanıtlıyor; ancak fizik yasaları çok değişkenli rasyonel ifadelerdir ($P_{\text{beam}} = \frac{I_{\text{sp}}^2 g_0^2 \dot{m}}{2\eta}$). Ayrıca kayan noktalı (`f64`) RK4 simülasyonu yuvarlama hatası içerir.
**Yapılacaklar:**
1. **Çok Değişkenli Kanonik Polinomlar (`MultiPolyQ`):** Lexicographic / Graded Reverse Lexicographic (grevlex) monom sıralamasıyla çok değişkenli rasyonel ifadelerin ($\frac{P(x_1,\dots,x_n)}{Q(x_1,\dots,x_n)}$, $Q \neq 0$ yan koşuluyla) tam cebirsel denklik ispatı.
2. **Garantili Aralık Aritmetiği (`Interval<f64> = [lo, hi]`):** Sayısal değerlendirmelerde tek bir `f64` yerine alt ve üst sınır aralığı $[a, b]$ taşınarak, IEEE-754 yuvarlama hatalarının güvenlik katsayısını (`safety_factor >= 1.35`) ihlal etmediğinin matematiksel olarak garanti altına alınması.
3. **Adaptif Dormand-Prince (RK45) Çözücüsü:** Sabit $\Delta t = 0.1\text{ s}$ adımı yerine, Max-Q ve kademe ayrılması gibi sert geçişlerde adım boyunu yerel kesme hatasına ($\|\mathbf{e}\| < \varepsilon_{\text{tol}}$) göre otomatik küçülten gömülü RK4(5) çifti.

---

### FAZ 4: Dikey Mühendislik Paketleri (Aerospace, İtki, Elektromanyetik ve Yapısal Mekanik) (Hedef: 6–10 Hafta)
**Sorun:** 20 kanonik yasa ve 8 malzeme kavramsal doğrulama için yeterlidir; endüstriyel ön tasarım aracı olmak için kapsamlı bir dikey kütüphane gerekir.
**Yapılacaklar:**
1. **150+ Kanonik Yasa (5 Dikey Modül):**
   * **Yörünge & Uçuş Mekaniği (35 Yasa):** Hohmann transferi, bi-eliptik transfer, $J_2$ basıklık pertürbasyonu, ağırlık-dönüşü (gravity turn) diferansiyel denklemleri, kaldırma/sürükleme polarları ($C_L, C_D(\text{Mach})$).
   * **İtki ve Termodinamik (35 Yasa):** De Laval süpersonik nozul izentropik akış denklemleri, yanma odası basıncı, Choked Flow, İyon/Hall-Effect itki, Lazer/Mikrodalga termal itki.
   * **Yapısal Mekanik & Kabuk Stabilitesi (30 Yasa):** İnce cidarlı silindirik kabukta eksenel Euler/NASA SP-8007 burkulma (buckling) kritik yükü, iç basınç stabilizasyonu, eğilme momenti, yorulma (fatigue).
   * **Aerotermodinamik & Isı Transferi (25 Yasa):** Sutton-Graves konvektif, Tauber-Sutton radyatif şok ısınması, ablatif ısı kalkanı kütle kaybı ($\dot{m}_{\text{ablation}} = q_{\text{net}} / H_{\text{eff}}$), rejeneratif soğutma.
   * **Elektromanyetik & Güç Sistemleri (25 Yasa):** Railgun/Coilgun Lorentz ve endüktans denklemleri, darbeli güç sığaç (capacitor bank) deşarjı, güneş paneli / lazer fotovoltaik güç bütçesi.
2. **50+ Sertifikalı Havacılık Malzemesi (`materials.json`):** Sıcaklığa bağlı akma dayanımı $\sigma_y(T)$ eğrileri, ısıl iletkenlik $k(T)$, özgül ısı $c_p(T)$ ve ablatif entalpi değerleri (PICA-X, Avcoat, Haynes 282, GRCop-42 bakır alaşımı, SiC/SiC CMC).

---

### FAZ 5: Otonom Mühendis Ajanı (MCP / Kokpit) ve Raporlama Motoru (Hedef: 10–12 Hafta)
**Sorun:** Motorun dış dünyadaki mühendisler veya yapay zeka ajanları tarafından tek komutla kullanılabilmesi.
**Yapılacaklar:**
1. **`citadel_math_core` MCP Sunucusu:** Claude, Cursor veya yerel LLM ajanlarının doğrudan çağırabildiği deterministik araçlar:
   * `math_verify_law(equation, dimensions)`
   * `math_synthesize(goal, inputs, domain_bounds)`
   * `math_simulate_trajectory(vehicle_spec)`
   * `math_optimize_design(constraints)`
2. **Otomatik Mühendislik Doğrulama Raporu Üretici:** Her tasarım için kullanılan denklem zincirini, boyutsal ispat tablosunu, RK4 uçuş profilini, Von Mises / burkulma güvenlik marjlarını ve erime sınırlarını içeren tek sayfalık denetlenebilir teknik föy üretimi.

---

## 🎯 İlk 7 Günlük Somut Eylem Planı (Hemen Başlanacak Adımlar)
1. **Gün 1–2:** `citadel_math_core` içine `Expr::parse_str` ekleyerek Rust çekirdeğinin tüm 20 yasayı doğrudan `database/canonical_laws.json` dosyasından okumasını sağlamak (Python-Rust çift yazımını bitirmek).
2. **Gün 3–4:** İnce cidarlı gövde denetimine **eksenel burkulma (Cylinder Axial Buckling - NASA SP-8007)** ve **ablatif ısı kalkanı (PICA-X / Karbon-Fenolik)** modelini eklemek (böylece 1703 °C şok sıcaklığında çıplak Al-Li 2195 erirken, kaç mm PICA-X kalkanla hayatta kaldığını hesaplamak).
3. **Gün 5–7:** `kmath.py design` / `citadel_math_core design` komutunu yazarak hipergraf + RK4 + malzeme testini **tek bir kapalı döngü optimizasyonda** birleştirmek.
