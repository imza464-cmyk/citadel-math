# Matematik Çekirdeği — Mimari Referans

**Tarih:** 26 Ekim 2026  
**Durum:** Tarihsel mimari tasarım referansı (çalışan `v0.1.0` uygulaması ve CI doğrulaması için bkz. [`README.md`](../README.md) ve [`citadel_math_core/`](../citadel_math_core/)).  
**Kapsam:** Bağımsız saf matematik, kanıt ve deterministik doğrulama projesi.

## 1. Amaç ve kesin ayrımlar

Amaç; matematiksel fonksiyonları, fonksiyon ailelerini, özellikleri ve dönüşüm kurallarını ortak bir yapıda temsil eden; bunlardan yeni fonksiyon adayları üretebilen bağımsız bir sistem kurmaktır.

- Matematik çekirdeğinin kendisi yapay zekâ değildir. İşleyişi açık kurallarla tanımlanacaktır.
- İnsanlar, matematik kaynakları ve yapay zekâlar tanım, özellik, ilişki ve kanıt adayları sağlayabilir. Yapay zekâ çıktısı kendiliğinden güvenilir kabul edilmez; bir modelin özel iç işleyişinin doğrudan çıkarılabildiği varsayılmaz.
- İlk hedef fiziksel icat, patent veya gerçek dünya başarısı değil; geçerli kurallarla fonksiyon adayları üretmektir.
- Uzun vadede geniş ve güvenilir bir matematik tabanı istenir. Milyonlarca bağımsız kayıt, ilk çalışan sürümün ön koşulu değildir; parametrik aileler tek tek örneklerden ayrı tutulur.
- “Yeni” başlangıçta mevcut kütüphaneye göre değerlendirilir. Dünya çapında özgünlük iddiası değildir.
- Deterministik çalışma, hatasızlık veya matematiksel tamlık garantisi değildir. Kullanılan mantık, varsayımlar ve uygulamanın doğruluğu ayrıca ele alınır.
- Mevcut formül arşivi kaynak katmanıdır; çalışan çekirdek veya makine tarafından denetlenmiş kanıt kütüphanesi değildir.

## 2. Mimari omurga

Bağlayıcı yapı: **ortak matematiksel temsil + türler + koşullar + denetlenebilir dönüşüm kuralları**.

```text
İnsan / Matematik kaynağı / Yapay zekâ
                  |
                  v
            Aday bilgi alanı
           (henüz güvenilmez)
                  |
                  v
      Ayrıştırma ve ortak temsil
    İfade + tür + tanım kümesi + koşullar
                  |
                  v
         Doğrulama çekirdeği <---------------------+
                  |                              |
                  v                              |
       Kabul edilmiş matematik tabanı             |
      Tanımlar + özellikler + kanıtlar             |
                  |                              |
                  v                              |
             Üretim motoru                       |
    Birleştirme / dönüştürme / bütçeli arama       |
                  |                              |
                  +---- Aday + türetim kaydı -----+
```

**Üretici önerir; doğrulayıcı denetler.** Üretim motoru, dış kaynak veya depolama katmanı bir kayda kendi başına “kanıtlandı” durumu veremez.

## 3. Katmanlar ve sorumluluk sınırları

| Katman | Sorumluluk | Yetki sınırı |
|---|---|---|
| Aday bilgi alanı | Kaynak, özgün içerik ve aday kayıtları tutmak | Gelen bilgiyi doğru ilan etmez |
| Ayrıştırıcı / ortak temsil | Metni yapılandırmak, tür ve kapsam denetimi yapmak | Doğal dil açıklamasını kanıt saymaz |
| Doğrulama çekirdeği | İzin verilen mantık ve kurallarla kanıt adımlarını denetlemek | Desteklemediği iddia için başarı bildirmez |
| Matematik tabanı | Tanım, önerme, kural ve bağımlılıkları saklamak | Tanımı, varsayımı ve teoremi karıştırmaz |
| Üretim motoru | Uyumlu girdilerden yeni ifade ve fonksiyon adayları oluşturmak | Kontrol edilmemiş adımları kabul edilmiş tabana taşımaz |
| Normalleştirme / tekrar denetimi | Desteklenen sınıfta ortak gösterim ve tekrar kontrolü | Genel fonksiyon eşitliğini çözdüğünü iddia etmez |
| Sonuç ve iz kaydı | Başarı, ret, belirsizlik ve kaynak sınırlarını kaydetmek | Kanıt ile sayısal örnek testini aynı etiketle sunmaz |

İlk sürümde bunlar mantıksal modül sınırlarıdır. Ayrı süreç, mikroservis, veritabanı veya dağıtık altyapı zorunluluğu oluşturmaz.

## 4. Ortak matematiksel temsil

Formüller yalnız LaTeX veya metin olarak saklanmamalıdır. İşlenebilir ana temsil, yapılandırılmış sembolik ifade olacaktır; metin ve LaTeX bunun görüntüleme biçimleridir.

Örnek:

$$f(x)=x^2+1$$

```text
Topla
├── Üs
│   ├── Değişken: x
│   └── Doğal sayı: 2
└── Tamsayı: 1
```

Başlangıç düğümleri: sabit, değişken, toplama, çarpma, doğal sayı üssü ve tanımlı fonksiyon uygulaması. Gereken kapsam oluşmadan yeni ifade türleri eklenmez.

Ortak alt ifadelerin paylaşımı daha sonra yönlü çevrimsiz grafik olarak ele alınabilir; ilk sürüm için zorunlu optimizasyon değildir. İfadelerin yapısal benzerliği, matematiksel eşitlik kanıtı yerine geçmez.

### Beş temel kayıt

| Kayıt | Asgari içerik |
|---|---|
| İfade | Düğüm türü, alt ifadeler, değişken kimlikleri ve tür bilgisi |
| Fonksiyon tanımı | Parametreler, girdi/çıktı türleri, tanım kümesi, gövde ve tanımlılık yükümlülükleri |
| Önerme | Açık varsayımlar altında ileri sürülen özellik veya ilişki |
| Dönüşüm kuralı | Girdi örüntüsü, önkoşullar, çıktı ve geçerlilik gerekçesi |
| Kanıt kaydı | Kullanılan varsayımlar, kural kimlikleri, ara adımlar ve sonuç |

Tanım ile hakkındaki iddia ayrıdır. Örneğin bir fonksiyonun tanımlanması; sürekli, birebir veya başka bir fonksiyona eşit olduğunun kanıtlandığı anlamına gelmez.

## 5. Fonksiyonları birbirine bağlayan sözleşmeler

### 5.1. Tür ve girdi–çıktı uyumu

Bir fonksiyonun çıktısı, onu alan fonksiyonun girdi türüyle uyumlu olmalıdır. Örtük ve denetlenmeyen tür dönüşümü yapılmaz. Gerekli dönüşüm varsa açık olarak ifade edilir.

### 5.2. Tanım kümesi ve yan koşullar

$$f:A\to B,\qquad g:C\to D$$

$g\circ f$ bileşkesinin bütün $A$ üzerinde tanımlı olması için $f(A)\subseteq C$ koşulu gerekir. Bu koşul gösterilemiyorsa sonuç otomatik kabul edilmez. Desteklenen dil izin verirse tanım kümesi, koşulu sağlayan girdilerle sınırlandırılmış yeni bir fonksiyon olarak açıkça kaydedilebilir.

Örnek:

$$r(x)=\frac{x^2-1}{x-1}=x+1,\qquad x\ne1$$

Koşul kaybedilmemelidir. Soldaki ifade, sağdaki polinomun bütün gerçek sayılardaki tanımıyla aynı tanım kümesine sahip değildir.

Bölme, kök veya logaritma gibi işlemler ilk polinom sürümüne dahil edilmez; bu örnek gelecekteki koşul taşıma gereksinimini açıklar.

### 5.3. Kanıtlanmış ilişkiler

İlişki türleri ihtiyaca göre eşitlik, bileşke, bir özelliğin korunması ve ileride türev veya terslik olabilir. Benzerlik, sayısal gözlem ve kanıtlanmış eşitlik ayrı etiketlenir.

Bir fonksiyonun ailesi ile o aileden üretilen örnekleri de ayrı kayıtlardır. Aile düzeyindeki bir teorem, koşulları sağlanan örneğe uygulanabilir; bu uygulama izlenebilir olmalıdır.

## 6. Küçük doğrulama çekirdeği

Çekirdeğin hangi mantık, temel tanımlar ve çıkarım kurallarını kabul ettiği açıkça belgelenmelidir. Bunlar geniş kütüphaneden ayrılır; her içe aktarılan formül temel aksiyoma dönüştürülmez.

- Temel aksiyom, tanım, kabul edilen varsayım ve türetilmiş teorem farklı statülerdir.
- Kanıt denetimi, sonucun kayıtlı varsayımlar ve kurallardan çıktığını kontrol eder.
- Her sonuç kullandığı varsayımlara ve kural sürümlerine bağlanır.
- Kural değişince ona bağlı kayıtlar yeniden denetlenmeden yeni sürümde geçerli sayılmaz.
- Sayısal örnek testi veya kaynakta “kanıtlanmış” yazması, makine tarafından denetlenmiş kanıt etiketi vermez.
- Hatalı bir kanıtın reddedilmesi ile bir iddianın yanlış olduğunun kanıtlanması farklıdır.

Bir yardımcı sadeleştirici veya arama yöntemi karmaşık olabilir; ancak sonuçları küçük denetleyicinin kabul ettiği adımlara dönmelidir. İlk prototip genel amaçlı bir teorem ispatlayıcı olmak zorunda değildir; desteklediği dar polinom kurallarını denetlemesi yeterlidir.

## 7. Üretim ve tekrar denetimi

### Başlangıç kapsamı önerisi

Tek değişkenli, tamsayı katsayılı polinomlar:

$$p(x)=\sum_{k=0}^{n}a_kx^k,\qquad n\in\mathbb{N}_0,\ a_k\in\mathbb{Z}$$

Bu, tüm matematiğin polinomlara indirgeneceği iddiası değildir; küçük ve sınanabilir bir ilk kapsamdır.

İlk üretim işlemleri:

1. Fonksiyon toplama.
2. Fonksiyon çarpma.
3. Fonksiyon bileşkesi.

### Örnek akış

$$f(x)=x^2,\qquad g(x)=x+1$$
$$h=f\circ g,\qquad h(x)=(x+1)^2=x^2+2x+1$$

```text
Kaynak tanımlar: f, g
Üretim işlemi: bileşke
Girdi / çıktı: gerçek sayı / gerçek sayı
Tanım kümesi: gerçek sayılar
Aday ifade: (x+1)²
Normal biçim: x²+2x+1
Kanıt izi: bileşke açılımı + denetlenmiş polinom dönüşümleri
Tekrar durumu: mevcut kabul edilmiş tabana göre değerlendirilir
```

Bu sınıfta katsayılar derece sırasıyla tutulabilir. Sondaki sıfır katsayıları temizlenmiş ortak bir gösterim önerilir; sıfır polinomu için tek gösterim seçilir. Tekrar denetimi, yalnız desteklenen sayı alanı ve fonksiyon yorumunda geçerlidir.

### Matematiksel sayı ile makine temsili

$\mathbb{Z}$ sınırsız tamsayıları ifade eder; sabit genişlikli depolama bu kümenin tamamını temsil etmez. İlk uygulama sabit genişlikli katsayı kullanırsa desteklenen sınırları açıkça bildirir. Taşma durumunda sarmalanmış sonucu matematiksel sonuç olarak kabul etmez; denetimli hata döndürür. Sınırsız tamsayı desteği ayrı bir tasarım kararıdır.

### Arama bütçesi

Üretim için azami derinlik, polinom derecesi, ifade boyutu, katsayı büyüklüğü ve işlem/adım bütçesi belirlenir. Adayların seçilme sırası sabitlenirse aynı girdi, kural sürümü ve bütçeyle tekrar üretilebilir sonuç hedeflenir.

## 8. Durum modeli

Tek bir “doğru/yanlış” bayrağı yerine farklı eksenler tutulmalıdır:

| Eksen | Örnek durumlar |
|---|---|
| İnceleme aşaması | Aday / denetlendi / yeniden denetim gerekiyor |
| Denetim sonucu | Kabul / reddedildi / kanıtlanamadı / desteklenmiyor / kaynak sınırı |
| Bilginin niteliği | Tanım / varsayım / denetlenmiş teorem / sayısal bulgu / hipotez |
| Yenilik | Kayıtla eşleşiyor / desteklenen kapsamda eşleşme bulunmadı / bilinmiyor |

“Kanıtlanamadı” yanlış demek değildir. “Eşleşme bulunmadı” da dünya çapında yeni demek değildir. Kabul edilmiş bir teorem, kullandığı varsayımlarla birlikte sunulur.

## 9. İlk sürümün geliştirme sırası ve kabul ölçütleri

1. **Veri sözleşmesi:** İfade, tanım, önerme, kural, kanıt ve durum kayıtları tanımlanır. Belirsiz girdiler reddedilir.
2. **Polinom temsili:** Katsayı listesi ve ortak gösterim uygulanır; sıfır polinomu ve katsayı sınırları test edilir.
3. **Üretim:** Toplama, çarpma ve bileşke adayları iz kaydıyla oluşturulur.
4. **Denetleme:** Desteklenen kurallarla sonuç tekrar kontrol edilir; bozuk adımların kabul edilmediği negatif testlerle sınanır.
5. **Tekrar ve bütçe:** Eşdeğer polinomlar tekrarlanmaz; bütçe bittiğinde açık durum döner.
6. **Kaynak aktarımı:** Küçük, elle gözden geçirilmiş bir kaynak kümesiyle başlatılır; sonra aynı sözleşmeyle genişletilir.

Asgari kabul örnekleri:

- $(x+1)^2$ ile $x^2+2x+1$ aynı polinom olarak tanınır.
- $f(x)=x^2$ ve $g(x)=x+1$ için iki bileşke sırasının farklı sonuçları doğru üretilir.
- Tür uyumsuzluğu, eksik önkoşul ve katsayı taşması başarıya çevrilmez.
- Hatalı kanıt kaydı reddedilir; sayısal örnekler kanıt diye kaydedilmez.
- Aynı girdi ve bütçede üretim sırası tekrar edilebilir; kütüphaneye kabul yolu yalnız doğrulayıcıdan geçer.

## 10. Proje ve kaynak sınırları

- Sistem dış işletim sistemi çekirdeklerinden bağımsızdır; çekirdek syscall, scheduler hotpath veya donanım özel veri yapılarına bağımlı tasarlanmaz.
- İşletim sistemi entegrasyonu bu tasarımın hedefi değildir; ileride istenirse ayrıca değerlendirilir.
- Langlands Programı ve kategori teorisi zorunlu altyapı değildir. Alanlar arası ilişki fikri ilham verebilir; otomatik evrensel çeviri veya keşif garantisi sunulmaz.
- Bu belge programlama dili, dış kütüphane, kanıt dili, dosya biçimi veya veritabanı seçmez; yeni bağımlılık yetkisi vermez.
- Kanıt üretme, kanıt denetleme, sayısal hesaplama ve fiziksel doğrulama ayrı işlerdir.

## 11. Mevcut dosyalarla ilişki

- Kaynak arşivi: [`formuller_ve_denklemler.md`](../formuller_ve_denklemler.md).
- Önceki düzeltme kaydı: [`logs/CHANGELOG_2026-10-26.md`](../logs/CHANGELOG_2026-10-26.md).
- Bu referans: [`docs/MATEMATIK_CEKIRDEGI_MIMARI_REFERANS.md`](./MATEMATIK_CEKIRDEGI_MIMARI_REFERANS.md).

Arşivdeki tanımlar otomatik olarak doğrulanmış tabana aktarılmayacaktır. Her aktarımda tanım kümesi, varsayımlar, kaynak ve doğrulama statüsü açıkça belirlenmelidir.

## 12. Bu kayıtla yapılan ve yapılmayan

**Yapılan:** Görüşmede netleşen hedeflerin ve mimari önerisinin yazılı referansa dönüştürülmesi.  
**Yapılmayan:** Çekirdeğin kodlanması, formal kanıt altyapısının kurulması, büyük veri aktarımı veya uçtan uca çalışmasının doğrulanması.

Bir sonraki uygulama adımı: bu tasarımdaki beş temel kayıt için küçük, kesin ve test edilebilir bir veri sözleşmesi oluşturmak. Bu belge o çalışmanın yerine geçmez.