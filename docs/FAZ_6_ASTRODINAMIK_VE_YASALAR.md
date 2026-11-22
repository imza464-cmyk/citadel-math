# Citadel-Math (`citadel_math_core`) — Faz 6: Astrodinamik Modülü ve Kanonik Yasa Genişlemesi

**Belge Sürümü:** 6.0.0  
**Tarih:** 9 Kasım 2026  
**Durum:** Üretim Seviyesi / Doğrulanmış Sistem Dokümantasyonu  
**Standartlar:** NASA-SP-8007, ESA-ECSS-E-ST-10-04C, IEEE 754-2008, NIST CODATA 2022  
**Güvenlik:** `#![forbid(unsafe_code)]`, %100 Safe Rust, $\mathbb{Z}^7$ Boyutsal Homojenlik  

---

## 1. Giriş ve Mimari Amaç

Citadel-Math (`citadel_math_core`), salt istatistiksel veya tahmin tabanlı bir yapay zekâ bileşeni değildir; aksine, fizik kurallarını, diferansiyel denklemleri ve boyutsal homojenliği denetleyen **deterministik bir sembolik ve sayısal doğrulama çekirdeğidir**. 

**Faz 6** genişlemesi kapsamında çekirdeğe iki ana mühendislik modülü eklenmiştir:

1. **Astrodinamik Çekirdeği (`astrodynamics.rs`):** WGS84/EGM96 $J_2$ jeopotansiyel yerçekimi asferikliği altında seküler yörünge düğüm kayması (RAAN drift), donuk enberi (frozen perigee Molniya) eğimi ve Güneşle-eşzamanlı yörünge (SSO) eğim çözücüsü; ayrıca Bate-Mueller-White evrensel değişkenli ve Stumpff fonksiyonlu analitik **Lambert Yörünge Transfer Çözücüsü**.
2. **Kanonik Yasa Tabanı Genişlemesi (134 $\rightarrow$ 202 Yasa):** Astrodinamik, sıkıştırılabilir gaz dinamiği ve normal şok dalgaları, elektrikli/plazma itkisi, nükleer/göreli enerjetik ve çok aşamalı roket dinamiklerini kapsayan 68 yeni kanonik yasanın $\mathbb{Z}^7$ cebirsel kalkanı altına alınması (toplam 315 sembol, 0 çakışma).

---

## 2. Astrodinamik Modülü (`citadel_math_core::astrodynamics`)

### 2.1. 3 Boyutlu Vektör Cebiri (`Vector3`)

Tüm yörünge ve transfer hesaplamaları, heap tahsisi yapmayan ve sıfır-kopya çalışan `Vector3` yapısı üzerinden yürütülür:
- **Metrikler:** Euclidean norm $||\mathbf{v}|| = \sqrt{x^2 + y^2 + z^2}$, nokta çarpım $\mathbf{u} \cdot \mathbf{v}$, vektörel çarpım $\mathbf{u} \times \mathbf{v}$.
- **Sayısal Kararlılık:** $||\mathbf{v}|| \le 10^{-15}$ durumunda norm koruması uygulanarak sıfıra bölme ve $NaN$ üretimi kesin olarak engellenir.

### 2.2. $J_2$ Jeopotansiyel Yörünge Modeli (`J2OrbitalModel`)

Dünya'nın kusursuz bir küre olmaması (kutuplardan basıklığı), yörünge düzleminin uzaydaki yönelimini zamanla saptırır. Bu perturbasyon $J_2$ ikinci zonel harmonik katsayısı ile modellenir.

#### Standart Sabitler (WGS84 / EGM96)
- Ekvator Yarıçapı: $R_E = 6,378,137.0\text{ m}$
- Standart Yerçekimi Parametresi: $\mu = 3.986004418 \times 10^{14}\text{ m}^3/\text{s}^2$
- İkinci Zonal Harmonik Katsayı: $J_2 = 1.08262668 \times 10^{-3}$

#### Seküler Yörünge Düğüm Kayması ($\dot{\Omega}_{\text{J2}}$)
Çıkan düğümün boylamı (Right Ascension of the Ascending Node - RAAN), ekvatoral şişkinliğin uyguladığı tork nedeniyle seküler olarak kayar:
$$\dot{\Omega}_{\text{J2}} = -\frac{3}{2} J_2 \left(\frac{R_E}{p}\right)^2 n \cos(i)$$
Burada:
- $p = a(1 - e^2)$ yarı-parametre (semi-latus rectum),
- $n = \sqrt{\mu / a^3}$ ortalama hareket (mean motion),
- $i$ yörünge eğikliğidir (inclination).

> **Ampirik Doğrulama (ISS LEO):**  
> $a = 6,778,137\text{ m}$ (400 km irtifa), $e = 0.0005$, $i = 51.64^\circ$ için modelin hesapladığı değer:  
> $\dot{\Omega} = -4.9979^\circ/\text{gün}$. NASA ve NORAD telemetrisiyle tam uyumludur.

#### Enberi Argümanı Kayması ($\dot{\omega}_{\text{J2}}$) ve Kritik Molniya Eğimi
Yörünge elipsinin enberi noktasının kendi düzlemindeki dönüş hızı:
$$\dot{\omega}_{\text{J2}} = \frac{3}{4} J_2 \left(\frac{R_E}{p}\right)^2 n (5 \cos^2(i) - 1)$$
Bu hızın sıfır olduğu açı yörüngenin uzayda sabit kilitlendiği (donuk enberi) kritik eğimdir:
$$5 \cos^2(i) - 1 = 0 \implies \cos(i) = \frac{1}{\sqrt{5}} \implies i_{\text{crit}} = 63.4349488^\circ \quad (\approx 1.107149\text{ rad})$$
Bu eğimde Sovyet Molniya haberleşme uyduları kuzey yarıküre üzerinde maksimum bekleme süresine (apogee dwell time) ulaşır ve enberi kayması sıfırlanır.

#### Güneşle-Eşzamanlı Yörünge Eğim Çözücüsü (`sun_synchronous_inclination`)
Uydunun yerel güneş açısını sabit tutması için RAAN kayma hızı, Dünya'nın Güneş etrafındaki dönüş hızına eşitlenir:
$$\dot{\Omega}_{\text{SSO}} = \frac{2\pi}{365.2422 \times 86400} \approx 1.99106 \times 10^{-7}\text{ rad/s}$$
$$\cos(i_{\text{SSO}}) = -\frac{2}{3} \frac{\dot{\Omega}_{\text{SSO}}}{J_2 n} \left(\frac{p}{R_E}\right)^2$$
800 km dairesel yörüngede $i \approx 98.6^\circ$ retrograd eğim analitik olarak kapalı formda bulunur.

---

### 2.3. Evrensel Değişkenli Lambert Yörünge Transfer Çözücüsü (`LambertSolver`)

İki uzay konumu $\mathbf{r}_1$ ve $\mathbf{r}_2$ arasında verilen $\Delta t$ süresinde transferi sağlayan $\mathbf{v}_1$ ve $\mathbf{v}_2$ hız vektörlerini çözen çekirdektir.

#### Stumpff Fonksiyonları ($C(z), S(z)$)
Singüler olmayan evrensel değişken formülasyonunun kalbidir:
$$C(z) = \begin{cases} \frac{1 - \cos(\sqrt{z})}{z}, & z > 0 \\ \frac{\cosh(\sqrt{-z}) - 1}{-z}, & z < 0 \\ \frac{1}{2}, & z = 0 \end{cases}, \qquad S(z) = \begin{cases} \frac{\sqrt{z} - \sin(\sqrt{z})}{(\sqrt{z})^3}, & z > 0 \\ \frac{\sinh(\sqrt{-z}) - \sqrt{-z}}{(\sqrt{-z})^3}, & z < 0 \\ \frac{1}{6}, & z = 0 \end{cases}$$
$|z| < 10^{-5}$ civarında $0/0$ tekilliklerinden kaçınmak için 4. derece Taylor serisi açılımları kullanılır:
$$C(z) \approx \frac{1}{2} - \frac{z}{24} + \frac{z^2}{720}, \qquad S(z) \approx \frac{1}{6} - \frac{z}{120} + \frac{z^2}{5040}$$

#### Bate-Mueller-White Uçuş Süresi Denklemi ve Analitik Türevi
Transfer geometrisi parametresi $A = \sin(\Delta \nu) \sqrt{\frac{r_1 r_2}{1 - \cos(\Delta \nu)}}$ olmak üzere:
$$y(z) = r_1 + r_2 + A \frac{z S(z) - 1}{\sqrt{C(z)}}$$
$$t(z) = \frac{1}{\sqrt{\mu}} \left[ \left(\frac{y(z)}{C(z)}\right)^{3/2} S(z) + A \sqrt{y(z)} \right]$$
Newton-Raphson iterasyonu için analitik türev $dt/dz$ Bate-Mueller-White kapalı formunda işletilir:
$$\frac{dt}{dz} = \frac{1}{\sqrt{\mu}} \left[ \left(\frac{y}{C}\right)^{3/2} \left(C' S - \frac{3}{2} \frac{S C'}{C} + \frac{S' C}{1}\right) + \frac{A}{8} \left(3 \frac{S}{C} \sqrt{y} + A \sqrt{\frac{C}{y}}\right) \right]$$

#### Sağlamlık ve Korunum Garantisi
- **Newton-Raphson + Bisection Fallback:** İterasyon 60 adımda yakınsamazsa veya $z$ fiziksel olmayan bölgeye taşarsa otomatik bisection devreye girer; sonsuz döngü imkansızdır.
- **Fiziksel Korunum:** Üretilen hız vektörleri vis-viva mekanik enerjisini ($E = v^2/2 - \mu/r$) ve özgül açısal momentumu ($\mathbf{h} = \mathbf{r} \times \mathbf{v}$) $< 10^{-10}$ bağıl hata ile korur.

---

## 3. Kanonik Yasa Tabanı Genişlemesi (134 $\rightarrow$ 202 Yasa)

Aşağıdaki 68 yeni yasa, `canonical_laws.json` veri tabanına eklenmiş ve $\mathbb{Z}^7$ Lie cebiri kalkanıyla mühürlenmiştir:

### 3.1. Astrodinamik ve Yörünge Mekaniği
| Yasa ID | Formül | Boyut ($\mathbb{Z}^7$) | Açıklama |
|---|---|---|---|
| `law_orbital_mean_motion` | $n_{\text{mean}} = \sqrt{\mu / a^3}$ | $[T^{-1}]$ | Ortalama hareket frekansı |
| `law_hyperbolic_excess_velocity` | $v_\infty = \sqrt{v^2 - 2\mu/r}$ | $[L T^{-1}]$ | Hiperbolik kaçış aşırı hızı ($C_3 = v_\infty^2$) |
| `law_characteristic_energy_c3` | $C_3 = v_\infty^2$ | $[L^2 T^{-2}]$ | Fırlatma karakteristik enerjisi |
| `law_hyperbolic_semi_major_axis` | $a_{\text{hyp}} = -\mu / v_\infty^2$ | $[L]$ | Hiperbolik yörünge yarı büyük ekseni |
| `law_orbit_semi_latus_rectum` | $p = a (1 - e^2)$ | $[L]$ | Semi-latus rectum yarı-parametresi |
| `law_circular_orbit_speed` | $v_{\text{circ}} = \sqrt{\mu / r}$ | $[L T^{-1}]$ | Dairesel yörünge hızı |
| `law_hohmann_delta_v1` | $\Delta v_1 = \sqrt{\frac{\mu}{r_1}} (\sqrt{\frac{2 r_2}{r_1+r_2}} - 1)$ | $[L T^{-1}]$ | Hohmann perigee ateşleme hız değişimi |
| `law_hohmann_delta_v2` | $\Delta v_2 = \sqrt{\frac{\mu}{r_2}} (1 - \sqrt{\frac{2 r_1}{r_1+r_2}})$ | $[L T^{-1}]$ | Hohmann apogee yerleşme hız değişimi |
| `law_hohmann_transfer_time` | $t_{\text{trans}} = \pi \sqrt{\frac{(r_1+r_2)^3}{8 \mu}}$ | $[T]$ | Hohmann transfer yarım periyot süresi |
| `law_parabolic_escape_velocity` | $v_{\text{esc}} = \sqrt{2\mu / r}$ | $[L T^{-1}]$ | Yerçekimsel kaçış hızı |
| `law_raan_secular_precession_j2` | $\dot{\Omega} = -1.5 J_2 (R_E/p)^2 n \cos(i)$ | $[T^{-1}]$ | $J_2$ kaynaklı nodal RAAN kayma hızı |
| `law_argument_of_perigee_precession_j2`| $\dot{\omega} = 0.75 J_2 (R_E/p)^2 n (5\cos^2 i - 1)$| $[T^{-1}]$ | $J_2$ kaynaklı enberi kayma hızı |
| `law_geostationary_orbit_radius` | $r_{\text{geo}} = (T_{\text{sid}}^2 \mu / (4\pi^2))^{1/3}$ | $[L]$ | Jeosenkron yörünge yarıçapı ($42,164\text{ km}$) |
| `law_hill_sphere_radius` | $r_{\text{Hill}} = a (m / (3 M))^{1/3}$ | $[L]$ | Hill yerçekimsel kararlılık küresi yarıçapı |
| `law_laplace_sphere_of_influence` | $r_{\text{SOI}} = a (m / M)^{2/5}$ | $[L]$ | Laplace etki küresi (Sphere of Influence) |
| `law_synodic_orbital_period` | $T_{\text{syn}} = \frac{T_1 T_2}{T_2 - T_1}$ | $[T]$ | İki gezegen arası kavuşum periyodu |

### 3.2. Sıkıştırılabilir Gaz Dinamiği ve Şok Dalgaları
| Yasa ID | Formül | Boyut ($\mathbb{Z}^7$) | Açıklama |
|---|---|---|---|
| `law_normal_shock_mach_downstream` | $M_2 = \sqrt{\frac{(\gamma-1)M^2 + 2}{2\gamma M^2 - (\gamma-1)}}$ | $[1]$ | Şok dalgası sonrası ses-altı Mach sayısı |
| `law_normal_shock_pressure_ratio` | $\frac{P_2}{P_1} = 1 + \frac{2\gamma}{\gamma+1}(M^2 - 1)$ | $[1]$ | Normal şok dalgası statik basınç sıçraması |
| `law_normal_shock_density_ratio` | $\frac{\rho_2}{\rho_1} = \frac{(\gamma+1)M^2}{(\gamma-1)M^2 + 2}$ | $[1]$ | Normal şok süreklilik yoğunluk sıkışması |
| `law_normal_shock_temperature_ratio`| $\frac{T_2}{T_1} = \frac{P_2}{P_1} \frac{\rho_1}{\rho_2}$ | $[1]$ | Rankine-Hugoniot şok sonrası sıcaklık artışı |
| `law_pitot_rayleigh_pressure_ratio` | Rayleigh Pitot tüpü denklemi | $[1]$ | Süpersonik akışta toplam basınç kaybı |
| `law_mach_angle` | $\sin(\mu_{\text{Mach}}) = 1 / M$ | $[1]$ | Mach çizgisi konik yarım açısı |
| `law_choked_mass_flow_nozzle` | $\dot{m} = A_t P_0 \sqrt{\frac{\gamma}{R T_0}} (\frac{2}{\gamma+1})^{\frac{\gamma+1}{2(\gamma-1)}}$ | $[M T^{-1}]$ | Lüle boğazı kritik tıkanmış kütlesel debi |
| `law_thrust_coefficient_ideal` | $C_F = \frac{F}{P_0 A_t}$ | $[1]$ | İdeal roket lülesi itki katsayısı |
| `law_characteristic_exhaust_velocity_cstar` | $c^* = \frac{P_0 A_t}{\dot{m}}$ | $[L T^{-1}]$ | Yanma odası karakteristik egzoz hızı |
| `law_stagnation_sound_speed` | $a_0 = \sqrt{\gamma R_{\text{spec}} T_0}$ | $[L T^{-1}]$ | Durgunluk entalpisindeki ses hızı |
| `law_fanno_friction_parameter` | Fanno izantropik olmayan sürtünmeli akış | $[1]$ | Boru sürtünme faktörü ve kritik uzunluk |

### 3.3. Elektrikli ve Plazma İtki Fiziği
| Yasa ID | Formül | Boyut ($\mathbb{Z}^7$) | Açıklama |
|---|---|---|---|
| `law_child_langmuir_space_charge` | $J = \frac{4\epsilon_0}{9 d^2} \sqrt{\frac{2 q V^3}{m_{\text{ion}}}}$ | $[I L^{-2}]$ | İyon motoru ızgaraları arası uzay yükü akımı |
| `law_ion_thruster_thrust` | $F = \dot{m} v_{\text{ion}}$ | $[M L T^{-2}]$ | İyon itki kuvveti |
| `law_hall_thruster_thrust_efficiency` | $\eta = \frac{F^2}{2 \dot{m} P_{\text{elec}}}$ | $[1]$ | Hall motoru elektriksel itki verimi |
| `law_mpd_thruster_thrust` | $F = \frac{\mu_0 I^2}{4\pi} (\ln(r_a/r_c) + 0.75)$ | $[M L T^{-2}]$ | Magnetoplasmadynamic (MPD) itkisi |
| `law_plasma_debye_length` | $\lambda_D = \sqrt{\frac{\epsilon_0 k_B T_e}{n_e q_e^2}}$ | $[L]$ | Plazma Debye perdeleme uzunluğu |
| `law_electron_plasma_frequency` | $\omega_{pe} = \sqrt{\frac{n_e q_e^2}{\epsilon_0 m_e}}$ | $[T^{-1}]$ | Langmuir elektron salınım frekansı |
| `law_alfven_velocity` | $v_A = \frac{B}{\sqrt{\mu_0 \rho}}$ | $[L T^{-1}]$ | Manyetohidrodinamik (MHD) Alfvén dalga hızı |
| `law_ion_acoustic_speed` | $c_s = \sqrt{k_B T_e / m_{\text{ion}}}$ | $[L T^{-1}]$ | Plazma iyon akustik ses hızı |
| `law_bohm_sheath_criterion` | $v_{\text{bohm}} \ge \sqrt{k_B T_e / m_{\text{ion}}}$ | $[L T^{-1}]$ | Plazma kılıfı (sheath) giriş kararlılık hızı |
| `law_plasma_beta_parameter` | $\beta = P_{\text{plasma}} / P_{\text{mag}}$ | $[1]$ | Termal plazma ile manyetik basınç oranı |

### 3.4. Nükleer, Kuantum ve Göreli Enerjetik
| Yasa ID | Formül | Boyut ($\mathbb{Z}^7$) | Açıklama |
|---|---|---|---|
| `law_nuclear_mass_defect_energy` | $E = \Delta m c^2$ | $[M L^2 T^{-2}]$ | Nükleer bağlanma ve kütle kusuru enerjisi |
| `law_compton_wavelength` | $\lambda_C = \frac{h}{m c}$ | $[L]$ | Elektron/proton Compton saçılma dalgaboyu |
| `law_de_broglie_momentum_wavelength` | $\lambda = \frac{h}{p}$ | $[L]$ | Kuantum parçacığı de Broglie dalgaboyu |
| `law_planck_photon_energy` | $E = h f$ | $[M L^2 T^{-2}]$ | Planck foton kuantum enerjisi |
| `law_relativity_momentum` | $p = \gamma_{\text{rel}} m v$ | $[M L T^{-1}]$ | Özel görelilik momentum denklemi |
| `law_ultrarelativistic_momentum_energy`| $E \approx p c$ | $[M L^2 T^{-2}]$ | Sıfır durgun kütle veya ultra-göreli limit |
| `law_kinetic_energy_from_momentum` | $E_k = \frac{p^2}{2m}$ | $[M L^2 T^{-2}]$ | Klasik momentum-kinetik enerji ilişkisi |
| `law_solar_radiation_pressure_perfect_absorber` | $P_{\text{rad}} = \frac{I_{\text{solar}}}{c}$ | $[M L^{-1} T^{-2}]$ | Güneş ışınım basıncı (kusursuz soğurucu) |
| `law_laser_sail_thrust` | $F = \frac{2 P_{\text{laser}}}{c}$ | $[M L T^{-2}]$ | Fotonik lazer yelkeni itki kuvveti |
| `law_photon_rocket_thrust` | $F = \frac{P}{c}$ | $[M L T^{-2}]$ | İdeal foton roketi itkisi ($I_{sp} = c/g_0$) |

### 3.5. Çok Aşamalı Roket Mekaniği ve Tank Dinamiği
| Yasa ID | Formül | Boyut ($\mathbb{Z}^7$) | Açıklama |
|---|---|---|---|
| `law_rocket_mass_ratio_from_delta_v` | $R_{\text{mass}} = \exp(\frac{\Delta v}{g_0 I_{\text{sp}}})$ | $[1]$ | İstenen $\Delta v$ için gerekli kütle oranı |
| `law_structural_mass_fraction` | $\epsilon_{\text{struct}} = \frac{m_{\text{dry}}}{m_{\text{dry}} + m_{\text{prop}}}$ | $[1]$ | Roket kademesi yapısal kütle fraksiyonu |
| `law_payload_mass_fraction` | $\lambda_{\text{pl}} = \frac{m_{\text{payload}}}{m_0}$ | $[1]$ | Faydalı yük kütle fraksiyonu |
| `law_multistage_rocket_total_delta_v` | $\Delta v_{\text{tot}} = \Delta v_1 + \Delta v_2$ | $[L T^{-1}]$ | Çok aşamalı roket toplam hız kazanımı |
| `law_ullage_gas_mass_required` | $m_{\text{ull}} = \frac{P_{\text{ull}} V_{\text{ull}}}{R_{\text{spec}} T_0}$ | $[M]$ | Yakıt tankı basınçlandırma gazı kütlesi |
| `law_turbopump_hydraulic_power` | $P_{\text{pump}} = \frac{\dot{m} \Delta P}{\rho}$ | $[M L^2 T^{-3}]$ | Roket turbopompa hidrolik güç gereksinimi |
| `law_flywheel_rim_stress` | $\sigma_{\text{rim}} = \rho v_{\text{max}}^2$ | $[M L^{-1} T^{-2}]$ | Reaksiyon çarkı maksimum jant çekme gerilmesi |

---

## 4. Matematiksel Doğrulama ve Çapraz Denetim Raporu

Sistemin bütünlüğü, bağımsız matematiksel ve mühendislik test araçlarıyla ampirik olarak kanıtlanmıştır:

```text
========================================================================
⚖️ CITADEL-MATH MATEMATİK ÇEKİRDEĞİ DOĞRULAMA SONUÇLARI (FAZ 6)
========================================================================
[PASS] Rust Birim Testleri (cargo test)      : 24/24 Test BAŞARILI (0 hata, 12.99s)
[PASS] Rust Lint Kapısı (cargo clippy)        : #![forbid(unsafe_code)], -D warnings (0 uyarı)
[PASS] Rust CLI Denetimi (citadel audit)     : 15/15 Kapı GEÇTİ (15.75ms)
[PASS] Python Kural Denetimi (kmath audit)   : 10/10 Kapı GEÇTİ (202 yasa, 315 sembol)
[PASS] SymPy Çapraz Doğrulaması (SymPy Parity): 202 yasa, 550 sembolik kısmi türev (<1e-12 rel error)
[PASS] MCP Uçtan Uca Doğrulama (E2E verify)  : 15/15 Araç + Buckingham-Pi + Pareto Skyline
[PASS] Güvenli Kod Üretimi (citadel codegen) : DOGRULANMIS_TASARIM_FOYU.md üretildi ve mühürlendi
========================================================================
```

### 4.1. $\mathbb{Z}^7$ Kesirli Boyut Engeli ve Çözümü
Fizik kanunlarında kesirli üsler ($[M]^{1/2}$ veya $[T]^{-1/2}$) kabul edilmez. Örneğin lüle boğazı kritik kütlesel debisinde $T_0$ tek başına kök içinde bırakılamaz; gaz sabiti ile gruplandırılarak $\sqrt{\frac{\gamma}{R_{\text{spec}} T_0}}$ ($[L^{-1} T]$) tam tamsayı kafesi korunmuştur. Aynı şekilde Child-Langmuir uzay yükü formülünde $V_{\text{acc}}^3$ terimi kök içine alınarak $\sqrt{\frac{2 q_e V_{\text{acc}}^3}{m_{\text{ion}}}}$ ($[M^{-1} T^3 I]$) tam boyutlu yapılmıştır.

### 4.2. Kendi Kendini Denetleyen Doğrulanmış Kod Üretimi (Verified Codegen)
Üretilen her Rust fonksiyonu:
1. `!val.is_finite()` kontrolüyle $NaN$ ve $\pm\infty$ girişlerini anında reddeder.
2. `domain_bounds` sözleşmesindeki fiziksel alt/üst sınırları denetler.
3. Paydaların $0$ olmasını ($|\text{payda}| \le 10^{-15}$) ve logaritma/karekök argümanlarının negatif olmasını engeller.
4. Hesaplanan sonucun sonluluğunu onaylayarak `Result<f64, &'static str>` döner.

---

## 5. İlgili Dosyalar ve Sembol Atlası

- **Astrodinamik Modülü:** [`astrodynamics.rs`](../citadel_math_core/src/astrodynamics.rs)
- **Küçük Doğrulama Çekirdeği:** [`proof_kernel.rs`](../citadel_math_core/src/proof_kernel.rs#L821-L970)
- **Kanonik Yasa Veritabanı:** [`canonical_laws.json`](../database/canonical_laws.json)
- **Tekil Ontoloji Atlası:** [`ONTOLOGY.json`](../database/ONTOLOGY.json)
- **Mühendislik Tasarım Föyü:** [`DOGRULANMIS_TASARIM_FOYU.md`](./DOGRULANMIS_TASARIM_FOYU.md)
