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
- [ ] Tam konu listesi — resmî 2026 belgesi doğrulandı; kaynak yılı tercihi ve metin doğrulaması bekliyor (docs/curriculum-research.md)

## Sonraki sürümler

1. Öğrenciden geri bildirim, konu bazlı plan şablonları ve kart düzenleme.
2. Haftalık analizler, deneme neti eğilimleri, gelişmiş tekrar algoritması.
3. Cloudflare Workers + D1, isteğe bağlı hesap ve cihazlar arası senkronizasyon; çakışma çözümü ve şema geçişleri.
4. Sunucu üzerinden yapay zekâ program önerileri. API anahtarı istemciye gönderilmez. İlk sürümde AI veya ücretli üyelik bulunmaz.
