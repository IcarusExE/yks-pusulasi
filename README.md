# YKS Pusulası

Canlı site: https://icarusexe.github.io/yks-pusulasi/

Rust + Leptos + WebAssembly ile hazırlanmış, verileri yalnızca tarayıcıdaki IndexedDB'de saklayan YKS çalışma alanı. Sunucu veya harici yapay zekâ servisi kullanmaz.

## Özellikler

- TYT + AYT; sayısal, eşit ağırlık, sözel ve dil alanları
- Günlük çalışmalar, haftalık tekrar, tikleme ve düzenlenebilir haftalık program taslağı
- Çalışmayı başka güne taşıma; haftalık seride yalnızca seçilen tekrarın taşınması
- Dört renk paleti ve açık/koyu cam teması
- Kronometre, 25 dakikalık Pomodoro ve odak geçmişi
- TYT/AYT/YDT netleri ve deneme geçmişi
- Deste bazlı hafıza kartları ve aralıklı tekrar
- JSON yedekleme/geri yükleme, çevrimdışı PWA

## Geliştirme

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk --version 0.21.14 --locked
trunk serve --address 0.0.0.0
```

```sh
cargo test --locked
npm ci --ignore-scripts
npm test
trunk build --release --public-url /yks-pusulasi/
```

## GitHub Pages

Depo **Settings → Pages → Source → GitHub Actions** olarak ayarlanmalıdır. `main` dalına gönderim test, WASM derlemesi ve Pages dağıtımını tetikler. Site adresi `https://<kullanıcı>.github.io/yks-pusulasi/` olur. Özel alan adı veya farklı depo adı için derleme public URL'sini değiştirin.

## Veri ve yedekler

Şema sürümü 1. 0.1.1 sürümü eski deneme ve odak kayıtlarına geriye uyumlu, kararlı kimlikler ekler. Ana durum tek IndexedDB işlemiyle yazılır; yazımlar sıraya alınır. Açılışta okuma veya doğrulama başarısızsa kayıtlar üzerine yazılmaz. Ayarlardan JSON yedek alın; tarayıcı depolamasının temizlenmesi, gizli oturumun kapanması veya site adresinin değişmesi verileri erişilemez hale getirebilir. Cihazlar arasında aktarım yedek dosyasıyla yapılır. Aynı tarayıcı profili içindeki pencereler BroadcastChannel üzerinden canlı eşitlenir. IndexedDB işlemi, son kaydı okuyup alan bazında birleştirerek yazar. Farklı kayıtlar ve farklı görev tamamlama tarihleri korunur; aynı alanın çakışmasında son kaydedilen değer, silme/düzenleme çakışmasında silme geçerlidir. BroadcastChannel olmayan tarayıcılarda pencereye yeniden odaklanıldığında kayıtlar okunur. Farklı cihazlar veya tarayıcı profilleri arasında sunucu üzerinden eşitleme yoktur.

PWA ilk çevrimiçi açılıştan ve service worker kurulumundan sonra çevrimdışı çalışır. Güncelleme, eski sürümün tüm sekmeleri kapandıktan sonra etkinleşir. Her sürümde `public/sw.js` içindeki cache sürümünü artırın. Cache adları uygulama yoluyla sınırlandırılır; başka uygulamaların cache kayıtlarına dokunulmaz.

Taşıma, tamamlanmamış çalışmalar için kullanılabilir. Haftalık bir görev taşındığında kaynak gün bir istisna olarak saklanır ve hedef güne tek seferlik bir çalışma eklenir; serinin diğer tekrarları ve tamamlanma geçmişi korunur. İki pencere aynı tekrarı aynı anda taşırsa tek hedef kayıt kullanılır; son kaydedilen hedef geçerli olur.

## Mimari ve sonraki adımlar

`src/model.rs`: Rust veri modelleri, doğrulama ve hesaplama kuralları. `src/main.rs`: Leptos arayüzü. `src/platform.rs`: tipli tarayıcı köprüsü. `public/storage.js`: IndexedDB, tarih, yedek ve PWA tarayıcı köprüsü. `public/style.css`: tema değişkenleri ve responsive görünüm. Rust durumu yerel depolama köprüsünden ayrılmıştır; Cloudflare Workers + D1 geçişinde kimlik, sürüm geçişleri ve çakışma çözümü içeren ayrı senkronizasyon katmanı eklenmelidir.

Yapay zekâ planlama bu sürümde etkin değildir. Gelecekte API anahtarını tarayıcıya koymadan Workers üzerinden sunulabilir. Ücretli üyelik/ödeme bulunmaz; “premium” görsel kaliteyi ifade eder.

Boş depo oluşturulup erişim verildikten sonra, GitHub CLI oturumu açıkken kaynakları göndermek ve Pages'i etkinleştirmek için:

```sh
bash scripts/publish.sh IcarusExE/yks-pusulasi
```

Bu komut mevcut bir Pages sitesinin yayın kaynağını GitHub Actions olarak değiştirir. Siteye erişim için Pages dağıtımının Actions ekranında tamamlanmasını bekleyin.
