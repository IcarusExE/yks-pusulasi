# Doğrulama — 0.2.0

## Otomatik kontroller

`cargo test --locked`: net hesabı, alanların soru toplamları, süre hesabı, haftalık tekrar ve geçersiz tarihler, yedek şema doğrulaması.

`npm ci --ignore-scripts && npm test`: fake-indexeddb ile aynı depoya bağlı bağımsız istemcilerin eşzamanlı ekleme/düzenleme/silmesi; farklı tamamlama günleri; son yazan kazanır kuralı; hızlı ardışık kayıtlar; eski kayıtların kimlikleri; yedekten tam değiştirme; yazım hatasından sonra yeniden deneme; yayın bildirimi; bozuk kaydın ham kopyasını alma. Service worker testleri izole çalışma ortamında dosya önbelleğini, offline geri dönüşü, sunucu hatasını ve başka uygulamaların cache kayıtlarının korunmasını sınar.

`trunk build --release --public-url /yks-pusulasi/`: WASM ve statik yayın dosyaları. GitHub Actions testleri ve derlemeyi tekrarlayıp Pages'e dağıtır.

## Gerçek tarayıcıda bekleyen kontroller

Tarayıcı aracı `the gaas-browser node_repl sandbox cannot be disabled` hatası verdi. Yukarıdaki testler gerçek tarayıcı, mobil görünüm veya kurulu PWA kontrolünün yerine geçmez.

- [ ] 360 / 390 / 768 / 1280 px genişliklerde yatay taşma ve dokunma hedefleri.
- [ ] Chrome/Android ve Safari/iPhone açık/koyu dört palette okunabilirlik.
- [ ] Program oluşturma, düzenleme, tikleme; yenileyip kayıtların korunması.
- [ ] İki pencere arasında görev ekleme/tikleme/silme ve tema değişiminin görünmesi.
- [ ] Kronometreyi başlatıp arka plana alma; geri dönüp süreyi kaydetme.
- [ ] TYT, dört alanın AYT/YDT netleri ve geçmiş silme.
- [ ] Kart oluşturma, cevap gösterme, tekrar aralığı seçme.
- [ ] JSON indirme ve geri yükleme dosya seçicisi; iptal; bozuk dosya.
- [ ] Kurulu PWA'yı internet kapalıyken açıp görev işaretleme.
- [ ] PWA güncellemesi: eski sürümün tüm pencerelerini kapatıp yeni sürümü açma.
- [ ] Klavyeyle modal açma, Tab / Shift+Tab sınırı, Escape ile kapatma.

## 0.1.2 ek kontroller

Otomatik: tek seferlik ve haftalık çalışma taşıma, tamamlanan çalışmada işlemi reddetme, geçersiz/same-date taşımanın veriyi değiştirmemesi, eski şemada istisna alanının varsayılanı, farklı ve aynı tekrarların pencereler arasında eşzamanlı taşınması.

Gerçek tarayıcıda taşıma penceresinin klavye ve telefon kullanımı hâlâ bekliyor.

## 0.2.0 ek kontroller

Otomatik: katalogdaki 18 dersin bütünlüğü ve kaynak sayfa aralıkları; benzersiz başlık kimlikleri; temel düzey matematik ve analiz konu kodları; Türkçe arama; kart düzenlemede tekrar tarihinin korunması; silinmiş/geçersiz kartı reddetme; boş hafta; taşınmış tekrarların haftalık sayımı; tarih ve test yapısına göre net karşılaştırması; diğer pencerenin kart tekrar ilerlemesinin düzenleme sırasında korunması.

Gerçek tarayıcıda bekleyenler: katalog filtreleri ve sayfalama, konudan taslak oluşturup değiştirme, yeni boş çalışmanın önceki taslağı kullanmaması, kart düzenleme penceresi, haftalık gezinme ve dar ekranlardaki tablo/grafik görünümü. Tarayıcı kısıtı nedeniyle bu maddeler doğrulanmış sayılmıyor.
