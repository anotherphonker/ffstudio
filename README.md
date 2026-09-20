<p align="center"><img src="assets/logo.png" width="160" alt="FF Studio logosu"></p>

# FF Studio — all-in-one ffmpeg GUI

Rust + egui ile yazılmış, **mavi tonlu koyu temalı**, sürükle-bırak destekli, ffmpeg GUI sarmalayıcısı.
ffmpeg'i **içine gömebilirsin** (all-in-one tek exe) ya da sistemdeki ffmpeg'i kullanır.

## Özellikler

- **Sürükle-bırak:** dosya veya klasörü pencereye at (klasördeki tüm medya taranır, alt klasörler dahil)
- **Dosya seçici:** çoklu dosya seçimi + klasör seçimi
- **Ses:** MP3 (V0 / V2 / 320-128 CBR), AAC/M4A, Opus, Vorbis, FLAC, WAV
- **Video:** H.264/MP4, H.265(MP4/MKV), AV1/MP4, VP9+Opus/WebM — CRF + hız + ses bitrate + küçültme
- **Hedef boyut sıkıştırma:** "bu videoyu 10 MB yap" — süreye göre bitrate otomatik hesaplanır
- **Kısa klip:** videodan başlangıç/bitiş süresiyle kesme (mm:ss)
- **GIF üretici:** video arasından döngülü GIF (genişlik + FPS ayarı, palet optimize)
- **Bölme:** videoyu N saniyelik parçalara hızlı bölme (kopya, yeniden kodlama yok)
- **Birleştirme:** listedeki tüm dosyaları TEK dosyada birleştirme (concat — kopya veya yeniden kodlama)
- **Resim:** JPEG, WebP, AVIF, PNG, BMP — kalite + küçültme
- **Kapak görseli çıkar:** ses dosyasındaki gömülü album art'ı (veya videonun ilk karesini) JPEG/PNG olarak kaydet
- **Remux:** sadece kapsül değiştir (stream copy — anında biter)
- **Otomatik:** dosyanın türüne göre en uygun preset (ses - MP3 V0, video - H.264, resim - JPEG)
- **Detay paneli:** dosyaya tıkla: kapsül, süre, codec, profil, çözünürlük, fps, pix_fmt, sample rate, kanal, stream başına bitrate
- **Bitrate & verim tablosu:** her dosya için kaynak kbps, hedef, tahmini çıktı boyutu ve yüzde fark
- **Komut önizlemesi:** tam ffmpeg komutu ekranda görünür
- **Paralel kuyruk (varsayılan):** CPU'na göre otomatik birden çok dosya aynı anda çevrilir; her biri için ilerleme çubuğu + hız (x2.5 gibi). Örnek: 4T i5-4590 → 2 paralel iş × 2 thread, 28T i7-14700F → 8 paralel iş × 3 thread. Tek dosya atarsan CPU'nun tamamı ona verilir (en yüksek tek-iş hızı). Kuyruk canlı olarak görünür: **N çalışıyor · N sırada · N bitti · N hata**. Sayı **Ayarlar → İşlemci**'den sabitlenebilir.
- **Özel argüman:** istediğin ek ffmpeg argümanı (örn: `-preset slow`). `-threads` otomatik eklenir; kendi `-threads` değerin varsa O kazanır (komutun sonunda gelir).
- **Log paneli:** her işlemin sonucu, hata detaylarıyla
- Üzerine yazma seçeneği; mevcut çıktı varsa varsayılan olarak atlanır
- **Kaynak dosya yönetimi:** başarılı dönüşümden sonra kaynağı **silebilir** veya **başka klasöre taşıyabilirsin** (çıktı doğrulanmadan asla dokunmaz)
- **Akıllı preset menüsü:** dosya türüne göre anlamsız preset'ler **gri (kullanılamaz)** olur — örn: FLAC attıysan video/resim preset'leri gridir
- **Dil:** **Türkçe (varsayılan) / İngilizce** — üst bardaki TR/EN ciplerinden anında geçiş, seçimi hatırlar
- **Yazı boyutu (DPI/zoom):** %75–%200 arası ölçekleme (varsayılan %125), yüksek DPI ekranda okunabilirlik için
- **Gerçek renkli emoji:** egui emoji'leri monokrom çizer; bu yüzden arayüzdeki emoji'ler **gömülü Twemoji PNG'si** olarak çizilir (flat, keskin). Dosyalar `assets/emoji/` altında **Unicode kod noktası (hex)** ile adlandırılmıştır (ör: `1f3ac.png` = 🎬, bayraklar tire ile `1f1f9-1f1f7.png` = 🇹🇷). Emoji'ler `include_bytes!` ile derlenirken .exe içine gömülür — Windows sürümünden ve sistem fontundan bağımsızdır.
- **Logo her yerde:** pencere başlığı, taskbar, üst panel ve Hakkında penceresi; Windows'ta dosya/exe ikonu da logodur (Gezgin'de, kısayolda, Alt-Tab'da görünür)
- **Tek pencere:** exe GUI alt sistemiyle derlenir — siyah konsol penceresi açılmaz
- **Hakkında:** üst bardaki ℹ️ butonu → yapıcı + sürüm + ffmpeg bilgisi
- **Tema:** koyu (varsayılan) / açık + **kendi vurgu rengin** (Ayarlar penceresinde; seçimin otomatik kaydedilir)
- **Genişletilebilir dosya listesi:** sol paneli kenarından tutup genişletebilirsin; dosya adları panele sığacak kadar kısalır, üzerine gelince tam ad görünür

## Gereksinimler (derleme için)

- Windows + **Rust** (MSVC aracı zinciri): https://rustup.rs (varsayılan "stable MSVC")
- All-in-one istiyorsan: **ffmpeg essentials zip'i** (aşağıda)

## Derleme

`build.bat` dosyasını çalıştır (çıktı yolunu gösterir) ya da elle:

```bat
build.bat
```

```bat
cargo build --release
```

Exe: `target\release\ffstudio.exe`

Derle + çalıştır: `build.bat run`

Hızlı test için debug:

```bat
cargo run
```

## Ayarlar (sağ üst)

Üst barın sağında (en sağdan sola):

- **ℹ️ Hakkında** — yapıcı, sürüm + ffmpeg bilgisi penceresi
- **🌐 🇹 / 🇺🇸** — dil bayrak cipleri (seçili olan dolgulu)
- **⚙️ Ayarlar** — tek pencerede her şey:
  - **İşlemci & Paralellik:** algılanan CPU adı + çekirdek/thread sayısı,
    "aynı anda kaç dosya" seçimi (Otomatik + 1–16). Otomatik = mantıksal thread'in yarısı (2–8).
  - **Dil:** Türkçe / English
  - **Tema:** koyu/açık görünüm, **yazı boyutu (zoom %75–%200)**, hazır renk tonları,
    R/G/B kaydırıcıları veya doğrudan hex kodu (örn: `2ec4b6`)

Varsayılan: **koyu mod + mavi vurgu** (`#5B9DFF`), Türkçe, %125 yazı boyutu, **Otomatik paralellik**.
Seçim `%LOCALAPPDATA%\eframe\FF Studio\eframe.conf.json` içine kaydedilir;
uygulama yeniden açılınca aynen geri gelir.

## ffmpeg (all-in-one) — 3 seçenek

### 1) Gömülü (önerilen): ffmpeg.zip — repoda HAZIR
`ffmpeg.zip` bu repoda proje kökünde duruyor; **ayrıca indirmene gerek yok.**
1. Repoyu klonla/indir (`ffmpeg.zip` içinde gelir)
2. `cargo build --release`

Derleme anında zip binary içine gömülür. İlk açılışta `%LOCALAPPDATA%\ffstudio\ffmpeg\` klasörüne
otomatik çıkarılır. Tek exe + ilk açılışta ~2-3 dk'lık çıkarma işlemi, sonra her şey hızlı.

### 2) exe yanında
`ffmpeg.exe` + `ffprobe.exe` dosyalarını `ffstudio.exe` yanına (veya yanındaki `ffmpeg\` klasörüne) koy.

### 3) PATH
ffmpeg PATH'te takılıysa (örn: `winget install Gyan.FFmpeg`) otomatik bulunur.

Not: AV1 için (libsvtav1) essentials build yeterli.

## Preset & verim kısa rehberi

| Preset | Tipik bitrate | Not |
|---|---|---|
| MP3 V0 | ~220 kbps | LAME'in en iyi VBR'i, 320 CBR'e neredeyse eşdeğer, ~%30 küçük |
| MP3 320 CBR | 320 kbps | MP3 tavanı, en büyük dosya |
| AAC VBR yüksek | ~220 kbps | M4A, MP3'e benzer kalite |
| Opus 256 | 256 kbps | Aynı boyutta MP3'ten daha iyi |
| H.264 CRF 23 | değişken | kaynağın ~%25-40'ı |
| H.265 CRF 28 | değişken | H.264'ten ~%40 daha verimli |
| AV1 CRF 30 | değişken | en verimli, ama yavaş |
| Hedef 10 MB | süreyle orantılı | 10 MB'lık video = ~280 kbps (3 dk için) |
| Remux | aynı | kodlama yok, saniyeler |
| Bölme (kopya) | aynı | parçalar toplamı ≈ kaynak, çok hızlı |
| Kapak çıkar (JPEG q2) | küçük | tek kare — album art / ilk video karesi |

CRF tabanlı video kodlayıcılar sabit bitrate kullanmadığından tablodaki "Tahmini" köşesi
bu preset'ler için "CRF?" der; ses preset'lerinde tahmin = bitrate x süre (gayet doğru).

## Proje yapısı

```
ffstudio/
  build.bat         # tek komut derleme (cargo yoksa rustup.rs uyarısı)
  upload_github.bat # tek komutla GitHub'a yükleme (yerel kalır; repo'ya girmez)
  .gitignore        # target/ ve *.exe git'e girmez; ffmpeg.zip REPODA
  ffmpeg.zip        # gömülü ffmpeg (all-in-one build girdisi)
  Cargo.toml
  build.rs          # ffmpeg.zip varsa embed_ffmpeg cfg'ini açar +
                    # Windows exe ikonunu (assets/icon.ico) exe'ye gömer
  assets/
    logo.png        # app logosu (üst panel + Hakkında + pencere ikonu)
    icon.ico        # Windows dosya/exe ikonu (Gezgin, taskbar, kısayol)
    emoji/          # gömülü Twemoji PNG'leri (kod noktası hex adı, ör: 1f3ac.png)
  src/
    lib.rs          # UI + uygulama durumu + paralel kuyruk + ayarlar + tema + dil
    main.rs         # masaüstü giriş noktası + pencere (GUI alt sistemi: çift pencere yok)
    ffmpeg.rs       # binary bulma/gömülü çıkarma, ffprobe, PARALEL worker pool
    cpu.rs          # CPU algılama (model adı + çekirdek/thread) → otomatik paralellik
    profiles.rs     # tüm preset'ler, argüman üretimi, boyut tahmini
    lang.rs         # Türkçe/İngilizce tüm arayüz metinleri
    emoji.rs        # Twemoji PNG'lerini doku olarak yükler + emoji yardımcıları
```

## GitHub'a yükleme

1. **İlk sefer (bir kere):** <https://github.com/new> adresinde adını
   **`ffstudio`** yap, **empty** repo olarak oluştur (README ekleme).
2. Bu klasörde **`upload_github.bat`**'i çalıştır — `git init`, `add`,
   `commit` ve `push`'u tek tek halleder. İlk itişte GitHub girişi
   ister (tarayıcı açılır ya da kullanıcı adı + *Personal Access Token*:
   <https://github.com/settings/tokens> → "repo" izni).
3. **Sonraki güncellemeler:** kodu değiştir → `upload_github.bat` → bitti.

Not: `assets/logo.png`'yi değiştirirsen exe ikonunu da tazele:
`python -c "from PIL import Image; Image.open('assets/logo.png').convert('RGBA').save('assets/icon.ico', sizes=[(16,16),(24,24),(32,32),(48,48),(64,64),(128,128),(256,256)])"`
(Pillow gerekir; ikon zaten repoda olduğu için normalde buna gerek yok.)

Not: `ffmpeg.zip` **repo'da durur** (repoyu indiren all-in-one build alır,
ayrıca ffmpeg indirmesi gerekmez). Derleme çıktısı `target/` ve `*.exe`
git'e girmez; `upload_github.bat` de yerel kalır (repoda bulunmaz).

## Hata ayıklama

- Uygulama açıldığında üst barda ffmpeg'in nereden bulundu yazıyor (gömülü / exe yanında / PATH)
- Log panelinde her işlem satır satır görünür; hata satırları kırmızı
- Derleme hatası alırsan: `cargo build --release` çıktısındaki ilk hata bloğunu kopyala
- ffmpeg hatası alırsan (örn: codec yok): logda hata mesajı + komut önizlemesinden kontrol et

## Kaynak (lisans)

- **Twemoji** emoji grafikleri: [CC-BY 4.0](https://creativecommons.org/licenses/by/4.0/) — © Twitter/Mozilla. Arayüzdeki renkli emoji'ler bu gömülü PNG'lerden gelir.
