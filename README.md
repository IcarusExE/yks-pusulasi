# YKS Pusulası

Canlı site: https://icarusexe.github.io/yks-pusulasi/

Rust + Leptos + WebAssembly ile hazırlanmış, verileri yalnızca tarayıcıdaki IndexedDB'de saklayan YKS çalışma alanı. Sunucu veya harici yapay zekâ servisi kullanmaz.

## Özellikler

- TYT + AYT; sayısal, eşit ağırlık, sözel ve dil alanları
- Günlük çalışmalar, haftalık tekrar, tikleme ve düzenlenebilir haftalık program taslağı
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
node --test tests/*.test.mjs
trunk build --release --public-url /yks-pusulasi/
```

## GitHub Pages

Depo **Settings → Pages → Source → GitHub Actions** olarak ayarlanmalıdır. `main` dalına gönderim test, WASM derlemesi ve Pages dağıtımını tetikler. Site adresi `https://<kullanıcı>.github.io/yks-pusulasi/` olur. Özel alan adı veya farklı depo adı için derleme public URL'sini değiştirin.

## Veri ve yedekler

Şema sürümü 1. Ana durum tek IndexedDB işlemiyle yazılır; yazımlar sıraya alınır. Açılışta okuma veya doğrulama başarısızsa kayıtlar üzerine yazılmaz. Ayarlardan JSON yedek alın; tarayıcı depolamasının temizlenmesi, gizli oturumun kapanması veya site adresinin değişmesi verileri erişilemez hale getirebilir. Cihazlar arasında aktarım yedek dosyasıyla yapılır. Aynı anda bir sekmede kullanın: çok sekmeli birleştirme bu sürümde yoktur.

PWA ilk çevrimiçi açılıştan ve service worker kurulumundan sonra çevrimdışı çalışır. Güncelleme, eski sürümün tüm sekmeleri kapandıktan sonra etkinleşir. Her sürümde `public/sw.js` içindeki cache adını artırın.

## Mimari ve sonraki adımlar

`src/model.rs`: Rust veri modelleri, doğrulama ve hesaplama kuralları. `src/main.rs`: Leptos arayüzü. `src/platform.rs`: tipli tarayıcı köprüsü. `public/storage.js`: IndexedDB, tarih, yedek ve PWA tarayıcı köprüsü. `public/style.css`: tema değişkenleri ve responsive görünüm. Rust durumu yerel depolama köprüsünden ayrılmıştır; Cloudflare Workers + D1 geçişinde kimlik, sürüm geçişleri ve çakışma çözümü içeren ayrı senkronizasyon katmanı eklenmelidir.

Yapay zekâ planlama bu sürümde etkin değildir. Gelecekte API anahtarını tarayıcıya koymadan Workers üzerinden sunulabilir. Ücretli üyelik/ödeme bulunmaz; “premium” görsel kaliteyi ifade eder.

Boş depo oluşturulup erişim verildikten sonra, GitHub CLI oturumu açıkken kaynakları göndermek ve Pages'i etkinleştirmek için:

```sh
bash scripts/publish.sh IcarusExE/yks-pusulasi
```

Bu komut mevcut bir Pages sitesinin yayın kaynağını GitHub Actions olarak değiştirir. Siteye erişim için Pages dağıtımının Actions ekranında tamamlanmasını bekleyin.
