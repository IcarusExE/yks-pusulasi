# YKS Pusulası · Sürüm planı

## 0.1 — Cihazda çalışan temel sürüm

- [x] Rust / Leptos CSR, WASM ve Trunk
- [x] TYT + AYT/YDT ve alan seçimi
- [x] Tarihli çalışmalar, günlük tikleme ve haftalık tekrar
- [x] Gün seçimi / süre ile düzenlenebilir kural tabanlı haftalık taslak
- [x] Günlük tamamlanan görevler, soru hedefleri ve odak dakikaları
- [x] Kronometre, Pomodoro ve oturum geçmişi
- [x] Alan bazlı net hesaplama ve deneme geçmişi
- [x] Hafıza desteleri, cevap gösterme ve aralıklı tekrar
- [x] Orman, okyanus, kiremit ve mürdüm paletleri; açık/koyu tema
- [x] IndexedDB ve doğrulanan JSON yedekleri
- [x] PWA manifesti, yerel ikonlar, çevrimdışı dosya önbelleği
- [x] GitHub Pages için test/derleme/dağıtım iş akışı
- [x] Hesaplama, tarih/tekrar, zaman, kayıt sırası ve çevrimdışı erişim testleri
- [ ] Gerçek tarayıcıda mobil/masaüstü akış ve görünüm doğrulaması — araç ortam hatası nedeniyle bekliyor
- [x] GitHub deposu ve Pages yayını — https://icarusexe.github.io/yks-pusulasi/

## 0.1.1 — Kayıt ve PWA güvenilirliği

- [x] Pencereler arasında alan bazlı canlı eşitleme ve çakışma kuralları
- [x] Eski deneme/oturum kayıtlarına geriye uyumlu kimlikler
- [x] Hatalı kayıtların ham yedeğini indirebilme
- [x] Tarih girişlerinde doğrulama ve haftalık görevlerde geçersiz tarihe karşı koruma
- [x] Uygulama yoluna özel PWA cache ve diğer uygulamaların cache kayıtlarını koruma
- [x] Çoklu istemci, eşzamanlı düzenleme/silme, kayıt hatası ve offline regresyon testleri

## 0.1.2 — Çalışmayı başka güne taşıma

- [x] Tek seferlik çalışmanın gününü değiştirme
- [x] Haftalık serinin yalnızca seçilen tekrarını taşıma
- [x] Tamamlanmış geçmişi koruma ve çift taşımayı engelleme
- [x] Birden fazla penceredeki taşıma işlemlerini birleştirme
- [x] Resmî 2026 kapsamı doğrulandı; kullanıcı yıl belirtilerek kullanılmasını onayladı (docs/curriculum-research.md)

## 0.2.0 — Kaynaklı konu kataloğu, kart düzenleme ve haftalık özet

- [x] 2026 resmî belgesindeki 18 ders için 867 ünite/tema/alt konu/beceri başlığı ve kaynak sayfaları
- [x] Türkçe arama, ders seçimi, sayfalama ve konudan düzenlenebilir çalışma taslağı
- [x] Kartın tekrar takvimini koruyarak deste/soru/cevap düzenleme
- [x] Haftalık görev, soru hedefi, odak süresi ve ders bazlı özet
- [x] Yalnızca karşılaştırılabilir denemeler arasında haftalık net farkı
- [x] Şema v1 ve mevcut kayıtlarla uyumluluk
- [ ] Gerçek tarayıcı ve telefon kontrolü — ortam aracı hatası sürüyor

## 0.3.0 — Konu ilerlemesi ve eksik konu önerisi

- [x] Konu başına öğrenci tarafından seçilen dört ilerleme seviyesi
- [x] Seçili ders/arama kapsamı için ilerleme sayıları ve durum filtresi
- [x] Yeni çalışmalar ile 2026 kaynak konuları arasında kalıcı bağlantı
- [x] Uygun gün/süre bütçesiyle kural tabanlı öneri taslağı ve seçerek onaylama
- [x] Tamamlanan/planlanmış konuları hariç tutma ve onayda yeniden kontrol
- [x] Tekrar döngüsü ve eşzamanlı onaylarda mükerrer öneri koruması
- [x] Konu ilerlemesini pencereler arasında birleştirme ve eski yedekleri okuma
- [ ] Gerçek mobil/PWA kontrolü — tarayıcı aracı erişim hatası devam ediyor

## Sonraki sürümler

1. Öğrenciden geri bildirim, konu bazlı plan şablonları ve kart düzenleme.
2. Haftalık analizler, deneme neti eğilimleri, gelişmiş tekrar algoritması.
3. Cloudflare Workers + D1, isteğe bağlı hesap ve cihazlar arası senkronizasyon; çakışma çözümü ve şema geçişleri.
4. Sunucu üzerinden yapay zekâ program önerileri. API anahtarı istemciye gönderilmez. İlk sürümde AI veya ücretli üyelik bulunmaz.
