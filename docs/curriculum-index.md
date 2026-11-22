# 2026 resmî kapsam indeksi

Kaynak: MEB Talim ve Terbiye Kurulu, 26 Kasım 2025 tarihli 2026 YKS konu ve kazanımları belgesi. Kontrol: 22 Kasım 2026.

PDF: https://ttkb.meb.gov.tr/meb_iys_dosyalar/2025_11/26164023_2026_yks.pdf

581 sayfa; SHA-256: `f52502743b6372e5df7de42d741b02a0b18eec087eac785b50a1fe3dd4a6d536`.

## Kapsam ve granülerlik

Katalog **ünite, tema, alt konu ve dil becerisi bloğu** düzeyindedir; bütün açıklamaları veya her bir alt kazanımı ayrı bir çalışma maddesi hâline getirmez. Bir başlığın içindeki sınırlar ve ayrıntılı kazanımlar için resmî PDF sayfası esas alınır. 2027 kapsamı olduğu iddia edilmez. Plan içindeki TYT/AYT dersi kullanıcı tarafından seçilir; bu, resmî bir oturum sınıflandırması değildir.

| Ders | Başlık | PDF sayfaları |
| --- | ---: | --- |
| Türk Dili ve Edebiyatı | 41 | 4–50 |
| Din Kültürü ve Ahlak Bilgisi | 20 | 51–74 |
| Tarih | 18 | 75–102 |
| T.C. İnkılap Tarihi ve Atatürkçülük | 8 | 103–111 |
| Coğrafya | 16 | 112–123 |
| Matematik | 66 | 124–154 |
| Fizik | 78 | 155–182 |
| Kimya | 77 | 183–206 |
| Biyoloji | 30 | 207–226 |
| Felsefe | 9 | 227–238 |
| Mantık | 4 | 239–246 |
| Sosyoloji | 6 | 247–257 |
| Psikoloji | 4 | 258–270 |
| İngilizce | 40 | 271–316 |
| Almanca | 28 | 317–348 |
| Fransızca | 28 | 349–380 |
| Arapça | 339 | 381–501 |
| Rusça | 55 | 502–581 |

Toplam: 867 başlık.

## Kaynak işleme ve denetim

- Matematik, fizik, kimya ve biyolojide numaralı alt konu başlıkları alınır. Matematiğin TD.11/TD.12 temel düzey bölümü de dâhildir. Sayısal alt başlık dizilerinde atlanan sıra olmadığı denetlendi.
- Edebiyat, tarih, inkılap, din, felsefe grubu ve coğrafyada belgede verilen ünite/sınıf çerçevesi korunur. Edebiyatın ortak okuma, yazma ve sözlü iletişim bölümleri ayrıca gösterilir.
- İngilizcede sınıfa göre 40 tema; Almanca ve Fransızcada düzeye göre 28'er tema bulunur.
- Arapça belge, tema yerine başlangıç yolu, sınıf, düzey ve beceri tabloları kullanır. Bu bloklar ve dil yapıları ayrı başlıklardır. Açıklama sayfaları ikinci bir kazanım bloğu sayılmaz. Dil yapılarında yalnızca sol sütundaki başlıklar alınır; sağ sütundaki örnekler başlık diye kopyalanmaz.
- Rusça tema adları özgün Kiril yazımıyla, ortak beceriler Türkçe başlıklarla gösterilir. Metin katmanının yan yana birleştirdiği p513 ve p572 başlıkları ayrıldı.
- Mantık/Sosyoloji/Psikoloji bölümlerindeki eski font kodlaması sınırlandırılmış bir dönüşümle çözüldü; özellikle Psikoloji'nin dört ünite başlığı kaynak görüntüsüyle kontrol edildi. Fizik p165'teki bağlam dışı 11.2.4 tekrarı yerine asıl p173 başlığı kullanıldı.
- Konu kimlikleri ders/düzey/başlık/sayfadan türetilir; sıralama değişikliği kimlikleri değiştirmez. Kaynak PDF değişirse üretici SHA-256 kontrolü nedeniyle durur ve yeniden inceleme gerekir.

## Yeniden üretme

PyMuPDF bulunan geliştirme ortamında resmî PDF indirildikten sonra:

```sh
python scripts/extract-curriculum.py /mutlak/yol/2026-yks.pdf data/curriculum-2026.json
cargo test --locked
```

PDF uygulamaya gömülmez. JSON listesi WASM içinde çevrimdışı çalışır; kaynak bağlantıları internet kullanır.
