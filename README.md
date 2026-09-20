<p align="center"><img src="gui/assets/logo.png" width="160" alt="FF Studio logosu"></p>

# FF Studio — all-in-one ffmpeg (Windows GUI + Termux TUI)

Tek çekirdek (`core`), iki istemci:

| İstemci | Ne için | Nerede çalışır |
|---|---|---|
| **GUI** (`gui`) | sürükle-bırak destekli masaüstü arayüz, ffmpeg exe içine gömülü (all-in-one) | Windows |
| **TUI** (`tui`) | klavye ile terminal arayüzü, sistemdeki ffmpeg'i kullanır | **Termux** (Android) ve her Linux terminal |

Her iki istemci de aynı `core` kütüphanesini kullanır: preset/argüman üretimi, ffprobe çözümleme, paralel
kuyruk mantığı ve TR/EN çeviriler **tek yerde** yazılıdır — birinde düzelen şey diğerinde de düzelir.

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
- **Gerçek renkli emoji:** egui emoji'leri monokrom çizer; bu yüzden arayüzdeki emoji'ler **gömülü Twemoji PNG'si** olarak çizilir (flat, keskin). Dosyalar `gui/assets/emoji/` altında **Unicode kod noktası (hex)** ile adlandırılmıştır (ör: `1f3ac.png` = 🎬, bayraklar tire ile `1f1f9-1f1f7.png` = 🇹🇷). Emoji'ler `include_bytes!` ile derlenirken .exe içine gömülür — Windows sürümünden ve sistem fontundan bağımsızdır.
- **Logo her yerde:** pencere başlığı, taskbar, üst panel ve Hakkında penceresi; Windows'ta dosya/exe ikonu da logodur (Gezgin'de, kısayolda, Alt-Tab'da görünür)
- **Tek pencere:** exe GUI alt sistemiyle derlenir — siyah konsol penceresi açılmaz
- **Hakkında:** üst bardaki ℹ️ butonu → yapıcı + sürüm + ffmpeg bilgisi
- **Tema:** koyu (varsayılan) / açık + **kendi vurgu rengin** (Ayarlar penceresinde; seçimin otomatik kaydedilir)
- **Genişletilebilir dosya listesi:** sol paneli kenarından tutup genişletebilirsin; dosya adları panele sığacak kadar kısalır, üzerine gelince tam ad görünür

## Termux TUI — kurulum ve kullanım

Android'de **Termux** içinde çalışır (APK yok, root yok — normal bir uygulama gibi kurulur).
Kendi Rust'ı ile derlenir; **NDK / cross-compile gerekmez**.

```bash
pkg update
pkg install rust ffmpeg termux-api

# depoyu klonla (ya da dosyaları /sdcard'dan bir klasöre kopyala)
git clone https://github.com/anotherphonker/ffstudio
cd ffstudio

# derle + PATH'e 'ffstudio' kısayolunu kur (ayrıntı: aşağıdaki "Derleme (Termux TUI)")
./build.sh

# artık her yerden:
ffstudio
```

`termux-setup-storage` komutunu bir kere çalıştırırsan telefonun depolamasına (`/sdcard`) erişebilirsin.

> Not: TUI tarafı **saf Rust** bağımlılıklar kullanır (ratatui + crossterm) — Termux'ta C derleyicisi
> ya da NDK gerekmez. Çok eski bir cargo `lock file version` hatası verirse `rm Cargo.lock` deyip
> tekrar derle; cargo uygun olanı kendi üretir.

**Kısayollar**

| Tuş | İş |
|---|---|
| `a` / `d` | dosya seç / klasör seç (klasör seçilirse içindeki tüm medya taranır, alt klasörler dahil) |
| `Space` | gezicide çoklu seçim, `Enter` klasöre girer, `1..9` hızlı klasörler (`/sdcard`, `Downloads`…) |
| `p` | preset menüsü (dosya türüne uymayan preset'ler **soluk/seçilemez**) |
| `c` / `Enter` | dönüşümü başlat (paralel kuyruk) |
| `o` / `r` / `w` / `M` | çıktı klasörü / kaynak dosya işlemi (aynen·sil·taşı) / üzerine yaz / taşıma klasörü |
| `C` / `g` / `G` / `m` | özel ffmpeg argümanı / kırpma başlangıç / bitiş / birleştirme adı |
| `+` `-` | seçili preset'in parametresi (CRF, bitrate, kalite, süre…) |
| `Tab` | panel değiştir (Dosyalar → Profil → Detay → Kuyruk → Log) |
| `t` / `l` / `s` | tema / dil (TR-EN) / ayarlar penceresi |
| `q` | çıkış (onay ister; çalışan işler için "durdurulacak" uyarısı verir) |
| `Ctrl+C` | **durdur ve çık** — çalışan ffmpeg süreçleri sonlandırılır (orphan kalmaz), terminal geri yüklenir |

**Özellikler (GUI ile aynı)**

- Ses: MP3 V0/V2/320-128 CBR, AAC/M4A, Opus, Vorbis, FLAC, WAV
- Video: H.264/MP4, H.265 (MP4/MKV), AV1/MP4, VP9+Opus/WebM — CRF + hız + ses bitrate + küçültme
- Resim: JPEG, WebP, AVIF, PNG, BMP — kalite + küçültme
- Hedef boyut, kısa klip (mm:ss), GIF (genişlik+FPS, palet optimize), N saniyelik parçalara bölme,
  birleştirme (kopya/yeniden kodlama), kapak çıkarma, remux, türe göre otomatik preset
- **Komut önizlemesi özdeştir:** ekranda görünen komut, çalıştırılan komutun ta kendisidir
  (ikisi de `core::profiles::build` + `core::ffmpeg::job_args` çıktısını kullanır)
- **Donanıma göre paralellik:** kuyruk canlı gösterir — *N çalışıyor · N sırada · N bitti · N hata*;
  ör. 4T cihaz → 2 paralel iş × 2 thread. Ayarlar'dan sabitlenebilir, `-threads` otomatik eklenir
  (kendi `-threads` değerinizi verirseniz o kazanır).
- Mevcut çıktı varsa **varsayılan olarak atlanır**
- Başarılı dönüşümden sonra kaynağı silme/taşıma — çıktı doğrulanmadan kaynağa **dokunulmaz**
- Çıktı yolu girdinin kendisi olacaksa (`mp4 → mp4` aynı klasör) `_donusen` ekli güvenli ad kullanılır; kaynak asla ezilmez
- `termux-wake-lock` ile ekran kapalıyken de dönüşüm sürer (`termux-api` kuruluysa)
- Log paneli (hatalar farklı renkte), TR/EN dil, terminal-güvenli 4 hazır tema
- **Panik koruması:** beklenmedik bir hata olsa bile terminal raw mode'da kalmaz (shell bozulmaz)
- **`--help` / `--version`:** `ffstudio --version` sürümü + bulunan ffmpeg'i + config yolunu gösterir
  (GUI'deki "Hakkında" panelinin CLI karşılığı)
- **`NO_COLOR`:** `NO_COLOR=1 ffstudio` → renkler kapanır, vurgular ters-video ile verilir
- **`XDG_CONFIG_HOME`:** ayar dosyası `$XDG_CONFIG_HOME/ffstudio-tui/config.json`, tanımlı değilse `~/.config/...`

**Ayar dosyası:** `~/.config/ffstudio-tui/config.json` (GUI'nin ayarlarından ayrıdır; dil, tema,
paralel iş sayısı, üzerine yazma, kaynak işlemi burada saklanır).

## Gereksinimler (derleme için)

- Windows + **Rust** (MSVC aracı zinciri): https://rustup.rs (varsayılan "stable MSVC")
- All-in-one istiyorsan: **ffmpeg essentials zip'i** (aşağıda)

## Derleme (Windows GUI)

`build.bat` dosyasını çalıştır (çıktı yolunu gösterir) ya da elle:

```bat
build.bat
```

```bat
cargo build --release -p gui
```

Exe: `target\release\ffstudio.exe`

> Bu depo bir **Cargo workspace**'tir: `core` (ortak mantık), `gui` (Windows arayüzü), `tui` (Termux).
> Kökte `cargo build --release` dersen üçü birden derlenir; sadece istediğini `-p <ad>` ile seçebilirsin.

Derle + çalıştır: `build.bat run`

Hızlı test için debug:

```bat
cargo run -p gui
```

## Derleme (Termux TUI)

Termux'ta çalıştırmak için:

1. `pkg install rust ffmpeg termux-api`
2. Repoyu klonla, kökte `./build.sh` çalıştır
3. Binary: `target/release/tui`

Derle + çalıştır: `./build.sh run`

Hızlı test için: `cargo run -p tui`

TUI, Windows sürümünden farklı olarak ffmpeg'i **gömmez** — sistemde
PATH'te bulunan ffmpeg'i kullanır (`pkg install ffmpeg` yeterli).

`./build.sh` ayrıca binary'yi PATH'e **kısayol** olarak kurar: Termux'ta `$PREFIX/bin/ffstudio`,
genel Linux'ta `~/.local/bin/ffstudio` (PATH'te değilse nasıl ekleneceğini söyler). Termux'ta
ek olarak `~/.shortcuts/ffstudio` oluşturulur — **Termux:Widget** kuruluysa ana ekrana tek dokunuş
ikonu olarak ekleyebilirsin.

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
2. `cargo build --release -p gui`  (ffmpeg.zip kökte dururken)

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
ffstudio/                 # Cargo workspace
  Cargo.toml              # [workspace] members = core, gui, tui
  build.bat               # Windows GUI derleme (cargo build --release -p gui)
  build.sh                # Termux/Linux TUI derleme (./build.sh  |  ./build.sh run)
  upload_github.bat       # tek komutla GitHub'a yükleme (yerel kalır; repo'ya girmez)
  .gitignore              # target/ ve *.exe git'e girmez; ffmpeg.zip REPODA
  ffmpeg.zip              # gömülü ffmpeg (all-in-one build girdisi)

  core/                   # ORTAK MANTIK (GUI + TUI aynı kodu çağırır)
    Cargo.toml            # lib adı: ffstudio_core
    build.rs              # ffmpeg.zip varsa embed_ffmpeg cfg'ini açar
    src/
      lib.rs              # modül listesi
      ffmpeg.rs           # binary bulma/gömülü çıkarma, ffprobe, PARALEL worker pool
      cpu.rs              # CPU algılama + paralellik planı (plan())
      profiles.rs         # tüm preset'ler, argüman üretimi, boyut tahmini
      lang.rs             # Türkçe/İngilizce tüm arayüz metinleri
      util.rs             # tarama (walk), boyut/süre biçimleme, çıktı doğrulama, taşıma

  gui/                    # WINDOWS MASAÜSTÜ (egui/eframe)
    Cargo.toml            # binary adı: ffstudio
    build.rs              # exe ikonu (assets/icon.ico) + sürüm bilgisi
    assets/
      logo.png            # app logosu (üst panel + Hakkında + pencere ikonu)
      icon.ico            # Windows dosya/exe ikonu (Gezgin, taskbar, kısayol)
      emoji/              # gömülü Twemoji PNG'leri (kod noktası hex adı)
    src/
      lib.rs              # arayüz + uygulama durumu + tema + dil (mantık core'dan)
      main.rs             # masaüstü giriş noktası (GUI alt sistemi: çift pencere yok)
      emoji.rs            # Twemoji PNG'lerini doku olarak yükler

  tui/                    # TERMUX / LINUX TUI (ratatui + crossterm)
    Cargo.toml            # binary adı: tui
    src/
      main.rs             # ince giriş noktası (wake-lock + çalıştır + wake-unlock)
      lib.rs              # terminal kurulumu, olay döngüsü, kısayollar
      app.rs              # durum, kuyruk planı, komut önizleme (core ile birebir)
      ui.rs               # ratatui çizimi (panel/popup/tablo), 4 tema
      browse.rs           # dizin gezici (çoklu seçim, hızlı klasörler)
      config.rs           # ~/.config/ffstudio-tui/config.json
    tests/
      e2e.rs              # gerçek ffmpeg ile: klasör tara → FLAC → MP3 V0 paralel
```

## GitHub'a yükleme

> Depo artık workspace: `core/`, `gui/`, `tui/` klasörleri ve kökteki `Cargo.toml` push edilir.
> `upload_github.bat` bu yapıyı otomatik tanır (`core/src/lib.rs` + `gui/src/lib.rs` kontrolü).

1. **İlk sefer (bir kere):** <https://github.com/new> adresinde adını
   **`ffstudio`** yap, **empty** repo olarak oluştur (README ekleme).
2. Bu klasörde **`upload_github.bat`**'i çalıştır — `git init`, `add`,
   `commit` ve `push`'u tek tek halleder. İlk itişte GitHub girişi
   ister (tarayıcı açılır ya da kullanıcı adı + *Personal Access Token*:
   <https://github.com/settings/tokens> → "repo" izni).
3. **Sonraki güncellemeler:** kodu değiştir → `upload_github.bat` → bitti.

Not: `gui/assets/logo.png`'yi değiştirirsen exe ikonunu da tazele:
`python -c "from PIL import Image; Image.open('gui/assets/logo.png').convert('RGBA').save('gui/assets/icon.ico', sizes=[(16,16),(24,24),(32,32),(48,48),(64,64),(128,128),(256,256)])"`
(Pillow gerekir; ikon zaten repoda olduğu için normalde buna gerek yok.)

Not: `ffmpeg.zip` **repo'da durur** (repoyu indiren all-in-one build alır,
ayrıca ffmpeg indirmesi gerekmez). Derleme çıktısı `target/` ve `*.exe`
git'e girmez; `upload_github.bat` de yerel kalır (repoda bulunmaz).

## Hata ayıklama

- Uygulama açıldığında üst barda ffmpeg'in nereden bulundu yazıyor (gömülü / exe yanında / PATH)
- Log panelinde her işlem satır satır görünür; hata satırları kırmızı
- Derleme hatası alırsan: `cargo build --release -p gui` çıktısındaki ilk hata bloğunu kopyala
- ffmpeg hatası alırsan (örn: codec yok): logda hata mesajı + komut önizlemesinden kontrol et

## Kaynak (lisans)

- **Twemoji** emoji grafikleri: [CC-BY 4.0](https://creativecommons.org/licenses/by/4.0/) — © Twitter/Mozilla. Arayüzdeki renkli emoji'ler bu gömülü PNG'lerden gelir.
