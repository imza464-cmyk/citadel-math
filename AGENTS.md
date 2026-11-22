# Copyright (c) 2026 Mekserina Karaosmanoğlu. Source-Available (Read & Run Only) — All Modification & Derivative Rights Reserved.
# CITADEL-MATH (`citadel_math_core` / `kmath-mcp`) — MÜHENDİSLİK KURALLARI VE DEĞİŞMEZLERİ
# Hedef: Bilimsel Keşif, Otomatize Formül Hasadı ve Boyutsal Doğrulama Ekosistemi

# ────────────────────────────────────────────────────────────────────────────
# 0. PROTOKOL VE ÖNCELİK HİYERARŞİSİ
# ────────────────────────────────────────────────────────────────────────────
# Bu dosya, Matematik Çekirdeği üzerinde çalışan tüm yapay zeka ajanları (Claude,
# GPT, Gemini vb.) ve insan mühendisler için BAĞLAYICI VE TAVİZSİZ MÜHENDİSLİK ANAYASASIDIR.
# Citadel-Math mühendislik standardındaki katı disiplin, bu projede "Matematiksel ve Fiziksel
# Tutarlılık" ekseninde aynen uygulanır.

# ────────────────────────────────────────────────────────────────────────────
# 1. TEMEL MİMARİ DEĞİŞMEZLER (INVARIANTS)
# ────────────────────────────────────────────────────────────────────────────

## DEĞİŞMEZ 1: BOYUTSAL MUTLAKLIK KALKANI (DIMENSIONAL INVARIANCE SHIELD)
- 7 Temel SI boyutu ([M] Kütle, [L] Uzunluk, [T] Zaman, [I] Akım, [Theta] Sıcaklık,
  [N] Madde Miktarı, [J] Işık Şiddeti) mutlak korunum kuralıdır.
- Bilgi tabanına giren HER denklemde Sol Taraf (LHS) ile Sağ Taraf (RHS) boyut vektörü
  %100 eşit olmak zorundadır: dim(LHS) == dim(RHS).
- Boyutsal denklik testinden geçmeyen hiçbir ifade, formül veya icat adayı
  bilgi tabanına kaydedilemez; "kanıtlandı" sayılamaz.

## DEĞİŞMEZ 2: TERMODİNAMİK VE DOĞA YASALARININ AŞILAMAZLIĞI
- Sistemde üretilen hiçbir formül doğanın 4 temel korunum yasasını ihlal edemez:
  1. Enerjinin Korunumu (Termodinamik 1. Yasa)
  2. Entropi Artışı İlkesi (Termodinamik 2. Yasa: dS >= 0, %100'ün üzerinde termal verim yasaktır)
  3. Çizgisel ve Açısal Momentumun Korunumu (Reaksiyonsuz itki / tepkisiz itme yasaktır)
  4. Özel Görelilik Nedensellik Sınırı (Hiçbir kütleli parçacık v >= c hızına ulaşamaz)
- Bu yasaları doğrudan veya dolaylı olarak ihlal eden kombinasyonlar derhal
  budanır (pruning) ve "Fiziksel Olarak İmkânsız" etiketiyle karantinaya alınır.

## DEĞİŞMEZ 3: ÇİFT KATMANLI EGEMENLİK (HAM ADAY VS. KANONİK BİLGİ TABANI)
- Sistem iki yalıtılmış dünyadan oluşur:
  * Katman 1 (Aday Havuzu / Raw Staging): İnternetten (arXiv, Wikipedia, web crawler)
    çekilen veya yapay zekanın ürettiği formüller "ŞÜPHELİ VE GÜVENİLMEZ" kabul edilir.
  * Katman 2 (Kabul Edilmiş Kanonik Taban / Canonical Core): Yalnızca boyut testini
    geçmiş, tekillik noktaları belirlenmiş ve ampirik sınır koşulları doğrulanmış
    denklemler buraya terfi edebilir. Üretici önerir, Doğrulayıcı onaylar.

## DEĞİŞMEZ 4: SAYISAL GERÇEKLİK VE SINIR KOŞULLARI ZORUNLULUĞU (NUMERICAL GROUNDING)
- Bir formül yalnızca harfler dizisi (a = b * c) olarak kalamaz.
- Her formülün:
  1. Tanım Kümesi (Domain & Constraints: payda != 0, karekök içi >= 0)
  2. Sınır Koşulları (Limitler: x -> 0, x -> inf durumunda fiziksel davranış)
  3. Diferansiyel Entegrasyon Davranışı (Runge-Kutta / ODE simülasyonunda patlayıp patlamadığı)
  açıkça tanımlanmak ve test edilmek zorundadır.

## DEĞİŞMEZ 5: SAHTE ZAFER VE HALÜSİNASYON YASAĞI (ANTI-PREMATURE CANONIZATION)
- Deterministik sistem mühendisliğinde olduğu gibi; birkaç düzine formül ekleyip "Dünya matematiğini çözdük",
  "Devasa icat yaptık" şeklinde edebi ve içi boş övgüler KESİNLİKLE YASAKTIR.
- Başarı metriği edebiyat değil; taranan gerçek denklem sayısı, çalışan diferansiyel
  simülasyonlar, doğrulanan malzeme sınırları ve ampirik hesaplama telemetrisidir.

## DEĞİŞMEZ 6: KANONİK ADLANDIRMA VE SAF TEKİLLİK KAYNAĞI (SSOT ONTOLOGY)
- Aynı fiziksel nicelik için farklı semboller (`v_e`, `c_e`, `u_exhaust`) kafaya göre
  kullanılamaz. Bütün değişkenler merkezi `ONTOLOGY.json` kütüphanesine bağlanır.
- Klon ve kopya kod yasaktır. Bir formül kütüphanede varsa yeniden türetilmez;
  kanonik ID'si üzerinden referans verilir.

## DEĞİŞMEZ 7: OTOMATİK MÜHENDİSLİK DENETİM KAPISI (CI / AUDIT MANDATE)
- Her kod veya veri tabanı değişikliğinden sonra `python kmath.py audit` çalıştırılır.
- Denetim kapısı:
  1. Boyutsal tutarsızlık
  2. Tanımsız değişken referansı (dangling symbols)
  3. Sıfıra bölme potansiyeli
  4. NIST sabitleri sapması
  tespit ederse derleme ve entegrasyon ANINDA DURDURULUR. Sıfır tolerans esastır.
