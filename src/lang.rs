//! Türkçe / İngilizce arayüz metinleri.

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Tr,
    En,
}

impl Lang {
    pub fn parse(s: &str) -> Lang {
        if s.trim().eq_ignore_ascii_case("en") {
            Lang::En
        } else {
            Lang::Tr
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Lang::Tr => "tr",
            Lang::En => "en",
        }
    }
}

// Bazi anahtarlar yalnizca belirli platform/derleme senaryolarinda kullanilir
// (orn: SrcEmbedded sadece embed_ffmpeg'de); dead-code uyarisini kilitle.
#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Key {
    // üst bar
    Subtitle,
    FfmpegMissing,
    // durum çubuğu
    UnitFile,
    UnitQueue,
    StRunning,
    StScanning,
    StIdle,
    // sol panel
    AddFile,
    AddFolder,
    Clear,
    DropHint,
    NoFiles,
    LogFolder,
    MediaFound,
    LogDropped,
    LogAdded,
    LogSkippedFile,
    // detay paneli
    DContainer,
    DDuration,
    DTotalBitrate,
    DVideo,
    DVideoBitrate,
    DAudio,
    DAudioBitrate,
    HzUnknown,
    UnitChannel,
    // 1 - profil
    SecProfile,
    LabelPreset,
    LabelOutput,
    OutSource,
    OutOther,
    PickOutFolder,
    Overwrite,
    LabelSrcAction,
    SrcKeep,
    SrcDelete,
    SrcMove,
    PickSrcFolder,
    SrcNote,
    CustomArgs,
    CustomArgsHint,
    CmdPreview,
    CmdPreviewMerge,
    // 2 - verim
    SecEff,
    EffEmpty,
    ColFile,
    ColType,
    ColSize,
    ColDur,
    ColSrcKbps,
    ColTarget,
    ColEst,
    ColDiff,
    TypeAudio,
    TypeVideo,
    TypeImage,
    CrfNote,
    CopyLabel,
    // 3 - kuyruk
    SecQueue,
    ConvertAll,
    Running,
    NoJobs,
    Queued,
    Done,
    WordRunning,
    WordError,
    SkippedPrefix,
    ErrorPrefix,
    LogQueue,
    QueueWillRun,
    QueueSkipped,
    QueueFinished,
    UnitFinished,
    UnitFailed,
    LogDone,
    UnitSecond,
    LogCommand,
    LogWarning,
    LogError,
    LogSource,
    LogOk,
    SkipExists,
    SkipInPlace,
    // kontroller
    Bitrate,
    Crf,
    CrfText,
    EncodeSpeed,
    SpeedNote,
    AudioInVideo,
    OpusFixed,
    TargetSize,
    TargetNote,
    Width,
    Fps,
    GifNote,
    SegmentLen,
    SplitNote,
    OutputName,
    Reencode,
    MergeNote,
    TrimStart,
    TrimEnd,
    TimeFormatHint,
    MaxWidth,
    Original,
    PillMax,
    PillRecommended,
    UnitMin,
    AudioHdr,
    VideoHdr,
    ImageHdr,
    OtherHdr,
    InfoAutoSwitched,
    // 5 - tema / genel
    SecTheme,
    Appearance,
    Dark,
    Light,
    Accent,
    ResetColor,
    Language,
    ZoomLabel,
    AboutBtn,
    AboutTitle,
    AboutDesc,
    AboutVersion,
    AboutBuilt,
    // ffmpeg
    FfmpegNotFound,
    SrcEmbedded,
    SrcExeSide,
    SrcPath,
    ExtraCover,
    ExtraVideo,
    ExtraAudio,
    ExtraStream,
    // profile hata mesajlari
    NeedAudio,
    NeedVideo,
    NeedImage,
    TargetSizeNoDur,
    TrimBadOrder,
    SplitZero,
    MergeBuildNote,
    // ek
    LogHdr,
    Quality,
    RemuxNote,
    VbrHigh,
    TrimEndHint,
    NoFfmpegStart,
    NoFilesToConvert,
    MergeNeedTwo,
    ConcatWriteFail,
    MergeDescRe,
    MergeDescCopy,
    MergeWillRun,
    SrcDeleted,
    SrcMoved,
    SrcDelFail,
    SrcMoveFail,
    OutNotVerifiedDel,
    OutNotVerifiedMove,
    MoveNoDir,
    SuspiciousFile,
    FfmpegMissingProbe,
    PickFilesTitle,
    PickFolderTitle,
    Close,
    QualityWord,
    ClipStartWord,
    ClipEndWord,
    SplitDesc,
    TargetDesc,
    MergeDescConcat,
    AutoWord,
    NoCover,
    CoverPng,
    CoverNote,
    // ayarlar
    SettingsBtn,
    SettingsTitle,
    SecCpu,
    CpuWord,
    CpuCores,
    CpuThreadsOnly,
    CpuParallel,
    CpuPlan,
    AboutBy,
}

/// Renk secenekleri icin isim (SWATCHES sirasiyla).
pub fn swatch_name(lang: Lang, i: usize) -> &'static str {
    const TR: &[&str] = &[
        "Mavi (varsayılan)", "Lacivert", "Turkuaz", "Yeşil", "Mor", "Kırmızı", "Turuncu", "Gri",
    ];
    const EN: &[&str] = &[
        "Blue (default)", "Navy", "Teal", "Green", "Purple", "Red", "Orange", "Gray",
    ];
    let t = match lang {
        Lang::Tr => TR,
        Lang::En => EN,
    };
    t.get(i).copied().unwrap_or("?")
}

/// Dinamik isme ({}) sahip mesajlar: format! ile doldurulur.
pub fn tr(lang: Lang, k: Key) -> &'static str {
    match k {
        // üst bar
        Key::Subtitle => match lang {
            Lang::Tr => "all-in-one ffmpeg arayüzü",
            Lang::En => "all-in-one ffmpeg interface",
        },
        Key::FfmpegMissing => match lang {
            Lang::Tr => "ffmpeg BULUNAMADI",
            Lang::En => "ffmpeg NOT FOUND",
        },
        // durum
        Key::UnitFile => match lang {
            Lang::Tr => "dosya",
            Lang::En => "file(s)",
        },
        Key::UnitQueue => match lang {
            Lang::Tr => "kuyruk",
            Lang::En => "queue",
        },
        Key::StRunning => match lang {
            Lang::Tr => "çalışıyor",
            Lang::En => "running",
        },
        Key::StScanning => match lang {
            Lang::Tr => "taranıyor",
            Lang::En => "scanning",
        },
        Key::StIdle => match lang {
            Lang::Tr => "beklemede",
            Lang::En => "idle",
        },
        // sol panel
        Key::AddFile => match lang {
            Lang::Tr => "Dosya ekle",
            Lang::En => "Add file",
        },
        Key::AddFolder => match lang {
            Lang::Tr => "Klasör ekle",
            Lang::En => "Add folder",
        },
        Key::Clear => match lang {
            Lang::Tr => "Temizle",
            Lang::En => "Clear",
        },
        Key::DropHint => match lang {
            Lang::Tr => "Dosya/klasörü pencereye sürükleyip bırakabilirsin.",
            Lang::En => "You can drag & drop files/folders onto the window.",
        },
        Key::NoFiles => match lang {
            Lang::Tr => "Henüz dosya yok. Ekle ya da sürükle-bırak yap.",
            Lang::En => "No files yet. Add some or drag & drop.",
        },
        Key::LogFolder => match lang {
            Lang::Tr => "[Klasör]",
            Lang::En => "[Folder]",
        },
        Key::MediaFound => match lang {
            Lang::Tr => "medya dosyası bulundu",
            Lang::En => "media files found",
        },
        Key::LogDropped => match lang {
            Lang::Tr => "[Sürükle-Bırak]",
            Lang::En => "[Drop]",
        },
        Key::LogAdded => match lang {
            Lang::Tr => "[Eklendi]",
            Lang::En => "[Added]",
        },
        Key::LogSkippedFile => match lang {
            Lang::Tr => "[Atıldı]",
            Lang::En => "[Skipped]",
        },
        // detay
        Key::DContainer => match lang {
            Lang::Tr => "Kapsül",
            Lang::En => "Container",
        },
        Key::DDuration => match lang {
            Lang::Tr => "Süre",
            Lang::En => "Duration",
        },
        Key::DTotalBitrate => match lang {
            Lang::Tr => "Toplam bitrate",
            Lang::En => "Total bitrate",
        },
        Key::DVideo => match lang {
            Lang::Tr => "Video",
            Lang::En => "Video",
        },
        Key::DVideoBitrate => match lang {
            Lang::Tr => "Video bitrate",
            Lang::En => "Video bitrate",
        },
        Key::DAudio => match lang {
            Lang::Tr => "Ses",
            Lang::En => "Audio",
        },
        Key::DAudioBitrate => match lang {
            Lang::Tr => "Ses bitrate",
            Lang::En => "Audio bitrate",
        },
        Key::HzUnknown => match lang {
            Lang::Tr => "Hz okunamadı",
            Lang::En => "Hz unreadable",
        },
        Key::UnitChannel => match lang {
            Lang::Tr => "kanal",
            Lang::En => "ch",
        },
        // 1 - profil
        Key::SecProfile => match lang {
            Lang::Tr => "1 - Dönüşüm Profili",
            Lang::En => "1 - Conversion Profile",
        },
        Key::LabelPreset => match lang {
            Lang::Tr => "Preset:",
            Lang::En => "Preset:",
        },
        Key::LabelOutput => match lang {
            Lang::Tr => "Çıktı:",
            Lang::En => "Output:",
        },
        Key::OutSource => match lang {
            Lang::Tr => "Kaynak klasöre",
            Lang::En => "To source folder",
        },
        Key::OutOther => match lang {
            Lang::Tr => "Farklı klasör...",
            Lang::En => "Other folder...",
        },
        Key::PickOutFolder => match lang {
            Lang::Tr => "Çıktı klasörü seç",
            Lang::En => "Choose output folder",
        },
        Key::Overwrite => match lang {
            Lang::Tr => "Çıktı zaten varsa üzerine yaz",
            Lang::En => "Overwrite existing output",
        },
        Key::LabelSrcAction => match lang {
            Lang::Tr => "Kaynak dosyalar:",
            Lang::En => "Source files:",
        },
        Key::SrcKeep => match lang {
            Lang::Tr => "Aynen kalsın",
            Lang::En => "Keep",
        },
        Key::SrcDelete => match lang {
            Lang::Tr => "Çevirince sil",
            Lang::En => "Delete after",
        },
        Key::SrcMove => match lang {
            Lang::Tr => "Taşı",
            Lang::En => "Move",
        },
        Key::PickSrcFolder => match lang {
            Lang::Tr => "Kaynak dosyaların taşınacağı klasör",
            Lang::En => "Folder to move source files to",
        },
        Key::SrcNote => match lang {
            Lang::Tr => "(sadece BAŞARILI dönüşümden sonra uygulanır; çıktı doğrulanmadan kaynak asla silinmez/taşıılmaz)",
            Lang::En => "(applied only after a SUCCESSFUL conversion; sources are never touched unless the output is verified)",
        },
        Key::CustomArgs => match lang {
            Lang::Tr => "Özel ffmpeg argümanı:",
            Lang::En => "Custom ffmpeg args:",
        },
        Key::CustomArgsHint => match lang {
            Lang::Tr => "(boş bırakılabilir, ör: -threads 4)",
            Lang::En => "(optional, e.g. -threads 4)",
        },
        Key::CmdPreview => match lang {
            Lang::Tr => "Komut önizlemesi (ilk dosya):",
            Lang::En => "Command preview (first file):",
        },
        Key::CmdPreviewMerge => match lang {
            Lang::Tr => "Komut önizlemesi (birleştirme):",
            Lang::En => "Command preview (merge):",
        },
        // 2 - verim
        Key::SecEff => match lang {
            Lang::Tr => "2 - Bitrate ve Verim",
            Lang::En => "2 - Bitrate & Efficiency",
        },
        Key::EffEmpty => match lang {
            Lang::Tr => "Dosya ekleyince burada kaynak bitrate, hedef, tahmini çıktı boyutu ve fark gösterilir.",
            Lang::En => "When you add files, source bitrate, target, estimated output size and difference appear here.",
        },
        Key::ColFile => match lang {
            Lang::Tr => "Dosya",
            Lang::En => "File",
        },
        Key::ColType => match lang {
            Lang::Tr => "Tür",
            Lang::En => "Type",
        },
        Key::ColSize => match lang {
            Lang::Tr => "Boyut",
            Lang::En => "Size",
        },
        Key::ColDur => match lang {
            Lang::Tr => "Süre",
            Lang::En => "Duration",
        },
        Key::ColSrcKbps => match lang {
            Lang::Tr => "Kaynak kbps",
            Lang::En => "Source kbps",
        },
        Key::ColTarget => match lang {
            Lang::Tr => "Hedef",
            Lang::En => "Target",
        },
        Key::ColEst => match lang {
            Lang::Tr => "Tahmini",
            Lang::En => "Est.",
        },
        Key::ColDiff => match lang {
            Lang::Tr => "Fark",
            Lang::En => "Diff",
        },
        Key::TypeAudio => match lang {
            Lang::Tr => "ses",
            Lang::En => "audio",
        },
        Key::TypeVideo => match lang {
            Lang::Tr => "video",
            Lang::En => "video",
        },
        Key::TypeImage => match lang {
            Lang::Tr => "resim",
            Lang::En => "image",
        },
        Key::CrfNote => match lang {
            Lang::Tr => "CRF tabanlı video kodlayıcılar sabit bitrate kullanmaz; boyut içeriğe göre değişir (H.264 CRF 23 tipik olarak kaynağın ~%25-40'ı olur). Ses preset'lerinde tahmin = bitrate x süre.",
            Lang::En => "CRF-based video encoders do not use a fixed bitrate; size depends on content (H.264 CRF 23 is typically ~25-40% of source). For audio presets, estimate = bitrate x duration.",
        },
        Key::CopyLabel => match lang {
            Lang::Tr => "kopya",
            Lang::En => "copy",
        },
        // 3 - kuyruk
        Key::SecQueue => match lang {
            Lang::Tr => "3 - Kuyruk",
            Lang::En => "3 - Queue",
        },
        Key::ConvertAll => match lang {
            Lang::Tr => "TÜMÜNÜ DÖNÜŞTÜR",
            Lang::En => "CONVERT ALL",
        },
        Key::Running => match lang {
            Lang::Tr => "Çalışıyor...",
            Lang::En => "Running...",
        },
        Key::NoJobs => match lang {
            Lang::Tr => "Henüz işlem yok. Dosya ekle, preset seç, 'TÜMÜNÜ DÖNÜŞTÜR'e bas.",
            Lang::En => "No jobs yet. Add files, pick a preset, press 'CONVERT ALL'.",
        },
        Key::Queued => match lang {
            Lang::Tr => "sırada",
            Lang::En => "queued",
        },
        Key::Done => match lang {
            Lang::Tr => "bitti",
            Lang::En => "done",
        },
        Key::WordRunning => match lang {
            Lang::Tr => "çalışıyor",
            Lang::En => "running",
        },
        Key::WordError => match lang {
            Lang::Tr => "hata",
            Lang::En => "error",
        },
        Key::SkippedPrefix => match lang {
            Lang::Tr => "atlandı:",
            Lang::En => "skipped:",
        },
        Key::ErrorPrefix => match lang {
            Lang::Tr => "HATA:",
            Lang::En => "ERROR:",
        },
        Key::LogQueue => match lang {
            Lang::Tr => "[Kuyruk]",
            Lang::En => "[Queue]",
        },
        Key::QueueWillRun => match lang {
            Lang::Tr => "işlenecek",
            Lang::En => "to process",
        },
        Key::QueueSkipped => match lang {
            Lang::Tr => "atlandı",
            Lang::En => "skipped",
        },
        Key::QueueFinished => match lang {
            Lang::Tr => "bitti",
            Lang::En => "done",
        },
        Key::UnitFinished => match lang {
            Lang::Tr => "tamamlandı",
            Lang::En => "finished",
        },
        Key::UnitFailed => match lang {
            Lang::Tr => "hata",
            Lang::En => "failed",
        },
        Key::LogDone => match lang {
            Lang::Tr => "[Tamam]",
            Lang::En => "[Done]",
        },
        Key::UnitSecond => match lang {
            Lang::Tr => "sn",
            Lang::En => "s",
        },
        Key::LogCommand => match lang {
            Lang::Tr => "[Komut]",
            Lang::En => "[Command]",
        },
        Key::LogWarning => match lang {
            Lang::Tr => "[Uyarı]",
            Lang::En => "[Warn]",
        },
        Key::LogError => match lang {
            Lang::Tr => "[Hata]",
            Lang::En => "[Error]",
        },
        Key::LogSource => match lang {
            Lang::Tr => "[Kaynak]",
            Lang::En => "[Source]",
        },
        Key::LogOk => match lang {
            Lang::Tr => "[ok]",
            Lang::En => "[ok]",
        },
        Key::SkipExists => match lang {
            Lang::Tr => "çıktı zaten var",
            Lang::En => "output already exists",
        },
        Key::SkipInPlace => match lang {
            Lang::Tr => "çıktı, kaynak dosyanın üzerine yazacaktı (aynı yol) - farklı çıktı klasörü ya da preset seç",
            Lang::En => "output would overwrite the source file (same path) - choose a different output folder or preset",
        },
        // kontroller
        Key::Bitrate => match lang {
            Lang::Tr => "Bitrate:",
            Lang::En => "Bitrate:",
        },
        Key::Crf => match lang {
            Lang::Tr => "CRF:",
            Lang::En => "CRF:",
        },
        Key::CrfText => match lang {
            Lang::Tr => "CRF  (küçük = kaliteli, büyük = ufak dosya)",
            Lang::En => "CRF  (small = quality, big = smaller file)",
        },
        Key::EncodeSpeed => match lang {
            Lang::Tr => "Kodlama hızı:",
            Lang::En => "Encoding speed:",
        },
        Key::SpeedNote => match lang {
            Lang::Tr => "(hız arttıkça dosya biraz büyür, kalite aynı kalır)",
            Lang::En => "(faster = slightly bigger file, same quality)",
        },
        Key::AudioInVideo => match lang {
            Lang::Tr => "Ses (video içindeki):",
            Lang::En => "Audio (in video):",
        },
        Key::OpusFixed => match lang {
            Lang::Tr => "Opus 160 kbps (WebM için sabit)",
            Lang::En => "Opus 160 kbps (fixed for WebM)",
        },
        Key::TargetSize => match lang {
            Lang::Tr => "Hedef boyut:",
            Lang::En => "Target size:",
        },
        Key::TargetNote => match lang {
            Lang::Tr => "(video bitrate dosya süresine göre otomatik hesaplanır; ses dahildir)",
            Lang::En => "(video bitrate is computed from duration automatically; audio included)",
        },
        Key::Width => match lang {
            Lang::Tr => "Genişlik:",
            Lang::En => "Width:",
        },
        Key::Fps => match lang {
            Lang::Tr => "FPS:",
            Lang::En => "FPS:",
        },
        Key::GifNote => match lang {
            Lang::Tr => "GIF'te ses olmaz; palet tek geçişte optimize edilir.",
            Lang::En => "GIF has no audio; palette is optimized in a single pass.",
        },
        Key::SegmentLen => match lang {
            Lang::Tr => "Parça süresi:",
            Lang::En => "Segment length:",
        },
        Key::SplitNote => match lang {
            Lang::Tr => "hızlı bölme (kopya, yeniden kodlama yok); parçalar anahtar karelere yakın kesilir: isim_part_001.mp4 ...",
            Lang::En => "fast split (copy, no re-encode); segments cut near keyframes: name_part_001.mp4 ...",
        },
        Key::OutputName => match lang {
            Lang::Tr => "Çıktı adı:",
            Lang::En => "Output name:",
        },
        Key::Reencode => match lang {
            Lang::Tr => "Yeniden kodla (codec/format farklıysa önerilir)",
            Lang::En => "Re-encode (recommended when codecs/formats differ)",
        },
        Key::MergeNote => match lang {
            Lang::Tr => "listedeki TÜM dosyalar sırayla TEK dosyaya birleştirilir; kopya modu aynı codec/format ister, farklıysa hata verir",
            Lang::En => "ALL files in the list are merged into ONE file in order; copy mode needs the same codec/format, otherwise it fails",
        },
        Key::TrimStart => match lang {
            Lang::Tr => "Başlangıç:",
            Lang::En => "Start:",
        },
        Key::TrimEnd => match lang {
            Lang::Tr => "Bitiş:",
            Lang::En => "End:",
        },
        Key::TimeFormatHint => match lang {
            Lang::Tr => "süre biçimi: ss | dd:ss | ss:dd:ss  (örn: 30, 1:05, 01:02:03); boş alan = dosyanın başı/sonu",
            Lang::En => "time format: ss | dd:ss | ss:dd:ss  (e.g. 30, 1:05, 01:02:03); empty = start/end of file",
        },
        Key::MaxWidth => match lang {
            Lang::Tr => "Maks. genişlik:",
            Lang::En => "Max width:",
        },
        Key::Original => match lang {
            Lang::Tr => "Orijinal",
            Lang::En => "Original",
        },
        Key::PillMax => match lang {
            Lang::Tr => "(maksimum)",
            Lang::En => "(max)",
        },
        Key::PillRecommended => match lang {
            Lang::Tr => "(önerilen)",
            Lang::En => "(recommended)",
        },
        Key::UnitMin => match lang {
            Lang::Tr => "dk",
            Lang::En => "min",
        },
        Key::AudioHdr => match lang {
            Lang::Tr => "SES ÇEVİRİMİ",
            Lang::En => "AUDIO",
        },
        Key::VideoHdr => match lang {
            Lang::Tr => "VIDEO",
            Lang::En => "VIDEO",
        },
        Key::ImageHdr => match lang {
            Lang::Tr => "RESİM",
            Lang::En => "IMAGE",
        },
        Key::OtherHdr => match lang {
            Lang::Tr => "DİĞER",
            Lang::En => "OTHER",
        },
        Key::InfoAutoSwitched => match lang {
            Lang::Tr => "[Bilgi] Seçili preset dosya türlerine uymuyor - 'Otomatik'e döndürüldü.",
            Lang::En => "[Info] Selected preset doesn't fit the file types - switched to 'Auto'.",
        },
        // 5 - tema / genel
        Key::SecTheme => match lang {
            Lang::Tr => "Tema",
            Lang::En => "Theme",
        },
        Key::Appearance => match lang {
            Lang::Tr => "Görünüm:",
            Lang::En => "Appearance:",
        },
        Key::Dark => match lang {
            Lang::Tr => "Koyu",
            Lang::En => "Dark",
        },
        Key::Light => match lang {
            Lang::Tr => "Açık",
            Lang::En => "Light",
        },
        Key::Accent => match lang {
            Lang::Tr => "Vurgu rengi:",
            Lang::En => "Accent color:",
        },
        Key::ResetColor => match lang {
            Lang::Tr => "Varsayılan renge dön",
            Lang::En => "Reset to default",
        },
        Key::Language => match lang {
            Lang::Tr => "Dil:",
            Lang::En => "Language:",
        },
        Key::ZoomLabel => match lang {
            Lang::Tr => "Yazı boyutu:",
            Lang::En => "Text size:",
        },
        Key::AboutBtn => match lang {
            Lang::Tr => "Hakkında",
            Lang::En => "About",
        },
        Key::AboutTitle => match lang {
            Lang::Tr => "Hakkında",
            Lang::En => "About",
        },
        Key::AboutDesc => match lang {
            Lang::Tr => "all-in-one ffmpeg GUI: ses, video ve resim dönüştürme; bitrate/verim tablosu, CPU'ya göre otomatik paralel kuyruk, sürükle-bırak, tema ve dil desteği.",
            Lang::En => "all-in-one ffmpeg GUI: audio, video and image conversion; bitrate/efficiency table, CPU-aware parallel queue, drag & drop, theming and language support.",
        },
        Key::AboutVersion => match lang {
            Lang::Tr => "Sürüm",
            Lang::En => "Version",
        },
        Key::AboutBuilt => match lang {
            Lang::Tr => "Rust + egui + ffmpeg ile yapıldı",
            Lang::En => "Built with Rust + egui + ffmpeg",
        },
        // ffmpeg
        Key::FfmpegNotFound => match lang {
            Lang::Tr => "ffmpeg/ffprobe bulunamadı. Seçenekler: 1) ffmpeg.zip (gyan.dev essentials) dosyasını proje köküne koyup yeniden derle (all-in-one) 2) ffmpeg.exe + ffprobe.exe'yi exe'in yanına koy 3) ffmpeg'i PATH'e ekle.",
            Lang::En => "ffmpeg/ffprobe not found. Options: 1) put ffmpeg.zip (gyan.dev essentials) in the project root and rebuild (all-in-one) 2) place ffmpeg.exe + ffprobe.exe next to the exe 3) add ffmpeg to PATH.",
        },
        Key::SrcEmbedded => match lang {
            Lang::Tr => "gömülü (program içinde)",
            Lang::En => "embedded (inside the program)",
        },
        Key::SrcExeSide => match lang {
            Lang::Tr => "exe yanında",
            Lang::En => "next to the exe",
        },
        Key::SrcPath => match lang {
            Lang::Tr => "PATH",
            Lang::En => "PATH",
        },
        Key::ExtraCover => match lang {
            Lang::Tr => "kapak görseli",
            Lang::En => "cover art",
        },
        Key::ExtraVideo => match lang {
            Lang::Tr => "video akışı",
            Lang::En => "video stream",
        },
        Key::ExtraAudio => match lang {
            Lang::Tr => "ses akışı",
            Lang::En => "audio stream",
        },
        Key::ExtraStream => match lang {
            Lang::Tr => "akışı",
            Lang::En => "stream",
        },
        // profile hata mesajlari
        Key::NeedAudio => match lang {
            Lang::Tr => "içinde ses akışı yok",
            Lang::En => "has no audio stream",
        },
        Key::NeedVideo => match lang {
            Lang::Tr => "içinde video akışı yok",
            Lang::En => "has no video stream",
        },
        Key::NeedImage => match lang {
            Lang::Tr => "bir resim değil; resim preset'leri sadece resim dosyalarına uygulanır",
            Lang::En => "is not an image; image presets only apply to image files",
        },
        Key::TargetSizeNoDur => match lang {
            Lang::Tr => "süre okunamadı, hedef boyut hesaplanamıyor",
            Lang::En => "duration could not be read, target size cannot be computed",
        },
        Key::TrimBadOrder => match lang {
            Lang::Tr => "bitiş zamanı başlangıçtan küçük olamaz",
            Lang::En => "end time cannot be smaller than start time",
        },
        Key::SplitZero => match lang {
            Lang::Tr => "parça süresi 0 olamaz",
            Lang::En => "segment length cannot be 0",
        },
        Key::MergeBuildNote => match lang {
            Lang::Tr => "Birleşme modu dosya listesine bakar, tek dosyaya değil (UI otomatik kurar)",
            Lang::En => "Merge mode works on the file list, not a single file (the UI builds it automatically)",
        },
        // ek
        Key::LogHdr => match lang {
            Lang::Tr => "4 - Log",
            Lang::En => "4 - Log",
        },
        Key::Quality => match lang {
            Lang::Tr => "Kalite:",
            Lang::En => "Quality:",
        },
        Key::RemuxNote => match lang {
            Lang::Tr => "kodlama yok, sadece kapsül değişir - anında biter",
            Lang::En => "no encoding, only the container changes - instant",
        },
        Key::VbrHigh => match lang {
            Lang::Tr => "VBR yüksek",
            Lang::En => "VBR high",
        },
        Key::TrimEndHint => match lang {
            Lang::Tr => "mm:ss (boş=sona)",
            Lang::En => "mm:ss (empty=end)",
        },
        Key::NoFfmpegStart => match lang {
            Lang::Tr => "ffmpeg yok, dönüşüm başlatılamadı.",
            Lang::En => "no ffmpeg, conversion not started.",
        },
        Key::NoFilesToConvert => match lang {
            Lang::Tr => "Çevrilecek dosya yok.",
            Lang::En => "No file to convert.",
        },
        Key::MergeNeedTwo => match lang {
            Lang::Tr => "Birleştirmek için en az 2 dosya gerekli.",
            Lang::En => "At least 2 files are needed to merge.",
        },
        Key::ConcatWriteFail => match lang {
            Lang::Tr => "[Hata] concat listesi yazılamadı: {e}",
            Lang::En => "[Error] could not write concat list: {e}",
        },
        Key::MergeDescRe => match lang {
            Lang::Tr => "Birleştirme (yeniden kodla)",
            Lang::En => "Merge (re-encode)",
        },
        Key::MergeDescCopy => match lang {
            Lang::Tr => "Birleştirme (kopya)",
            Lang::En => "Merge (copy)",
        },
        Key::MergeWillRun => match lang {
            Lang::Tr => "birleştirilecek",
            Lang::En => "will be merged",
        },
        Key::SrcDeleted => match lang {
            Lang::Tr => "silindi",
            Lang::En => "deleted",
        },
        Key::SrcMoved => match lang {
            Lang::Tr => "taşındı",
            Lang::En => "moved",
        },
        Key::SrcDelFail => match lang {
            Lang::Tr => "kaynak silinemedi: {name} ({e})",
            Lang::En => "could not delete source: {name} ({e})",
        },
        Key::SrcMoveFail => match lang {
            Lang::Tr => "kaynak taşınamadı: {name} ({e})",
            Lang::En => "could not move source: {name} ({e})",
        },
        Key::OutNotVerifiedDel => match lang {
            Lang::Tr => "çıktı doğrulanamadı, kaynak silinmedi",
            Lang::En => "output not verified, source was not deleted",
        },
        Key::OutNotVerifiedMove => match lang {
            Lang::Tr => "çıktı doğrulanamadı, kaynak taşınmadı",
            Lang::En => "output not verified, source was not moved",
        },
        Key::MoveNoDir => match lang {
            Lang::Tr => "Taşıma için klasör seçilmedi - kaynak aynen bırakıldı.",
            Lang::En => "No folder chosen for moving - source was left untouched.",
        },
        Key::SuspiciousFile => match lang {
            Lang::Tr => "şüpheli dosya - ses bilgisi okunamadı (bozuk ya da yanlış uzantılı olabilir); dönüşüm hata verebilir",
            Lang::En => "suspicious file - audio info could not be read (may be corrupt or mis-extended); conversion may fail",
        },
        Key::FfmpegMissingProbe => match lang {
            Lang::Tr => "ffmpeg yok, dosyalar taranamadı.",
            Lang::En => "no ffmpeg, files could not be probed.",
        },
        Key::PickFilesTitle => match lang {
            Lang::Tr => "Medya dosyası seç (çoklu seçim yapabilirsin)",
            Lang::En => "Select media files (multi-select possible)",
        },
        Key::PickFolderTitle => match lang {
            Lang::Tr => "Medya klasörü seç",
            Lang::En => "Select media folder",
        },
        Key::Close => match lang {
            Lang::Tr => "Kapat",
            Lang::En => "Close",
        },
        Key::QualityWord => match lang {
            Lang::Tr => "kalite",
            Lang::En => "quality",
        },
        Key::ClipStartWord => match lang {
            Lang::Tr => "baş",
            Lang::En => "start",
        },
        Key::ClipEndWord => match lang {
            Lang::Tr => "son",
            Lang::En => "end",
        },
        Key::SplitDesc => match lang {
            Lang::Tr => "{} sn'lik parçalar (kopya)",
            Lang::En => "{}s segments (copy)",
        },
        Key::TargetDesc => match lang {
            Lang::Tr => "Hedef ~{} MB",
            Lang::En => "Target ~{} MB",
        },
        Key::MergeDescConcat => match lang {
            Lang::Tr => "Birleştirme (concat)",
            Lang::En => "Merge (concat)",
        },
        Key::AutoWord => match lang {
            Lang::Tr => "Otomatik",
            Lang::En => "Auto",
        },
        Key::NoCover => match lang {
            Lang::Tr => "içinde kapak görseli yok",
            Lang::En => "has no embedded cover art",
        },
        Key::CoverPng => match lang {
            Lang::Tr => "PNG (lossless) olarak çıkar",
            Lang::En => "Extract as PNG (lossless)",
        },
        Key::CoverNote => match lang {
            Lang::Tr => "ses dosyasındaki gömülü album art (kapak) resim dosyasına çevrilir; videolarda ilk kare alınır",
            Lang::En => "the embedded album art in audio files is saved as an image; for video the first frame is taken",
        },
        // ayarlar
        Key::SettingsBtn => match lang {
            Lang::Tr => "Ayarlar",
            Lang::En => "Settings",
        },
        Key::SettingsTitle => match lang {
            Lang::Tr => "Ayarlar",
            Lang::En => "Settings",
        },
        Key::SecCpu => match lang {
            Lang::Tr => "İşlemci ve Paralellik",
            Lang::En => "Processor & Parallelism",
        },
        Key::CpuWord => match lang {
            Lang::Tr => "İşlemci",
            Lang::En => "CPU",
        },
        Key::CpuCores => match lang {
            Lang::Tr => "{p} çekirdek / {n} mantıksal thread",
            Lang::En => "{p} cores / {n} logical threads",
        },
        Key::CpuThreadsOnly => match lang {
            Lang::Tr => "{n} mantıksal thread",
            Lang::En => "{n} logical threads",
        },
        Key::CpuParallel => match lang {
            Lang::Tr => "Paralel iş (aynı anda kaç dosya çevrilsin):",
            Lang::En => "Parallel jobs (files converted at once):",
        },
        Key::CpuPlan => match lang {
            Lang::Tr => "→ sonraki çalıştırma: {w} paralel iş × {t} thread/iş",
            Lang::En => "→ next run: {w} parallel jobs × {t} threads/job",
        },
        Key::AboutBy => match lang {
            Lang::Tr => "Yapıcı",
            Lang::En => "Made by",
        },
    }
}
