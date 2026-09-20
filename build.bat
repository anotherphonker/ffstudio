@echo off
setlocal EnableExtensions
title FF Studio - build
rem ============================================================
rem  FF Studio build scripti
rem  - cargo build --release -p gui (workspace: masaustu GUI) calistirir
rem  - ffmpeg.zip ZORUNLU: all-in-one (gomulu ffmpeg) icin kullanilir
rem  Kullanim: build.bat            = sadece build
rem           build.bat run         = build + calistir
rem  Not: Termux TUI icin: cargo build --release -p tui
rem ============================================================

set "BUILD_ONLY=1"
if /i not "%~1"=="run" set "BUILD_ONLY=0"

echo.
echo  FF Studio build
echo  =================

where cargo >nul 2>nul
if errorlevel 1 goto :nocargo
cargo --version

if exist "%~dp0ffmpeg.zip" goto :zipvar
echo.
echo  [HATA] ffmpeg.zip bu klasorde YOK.
echo         GUI gomulu (all-in-one) ffmpeg tasarimini kullanir; bu dosya ZORUNLU.
echo         Cozum: gyan.dev essentials zip'ini bu klasore ffmpeg.zip adiyla koy.
echo         Not: Termux TUI bu dosyaya ihtiyac duymaz - ./build.sh ile derlenir.
echo.
pause
exit /b 1

:zipvar
echo  [ok] ffmpeg.zip bulundu - all-in-one build yapilacak.
echo.

cargo build --release -p gui
set "rc=%errorlevel%"
if not "%rc%"=="0" (
    echo.
    echo  [HATA] Build basarisiz oldu. Yukaridaki hata mesajlarina bak.
    pause
    exit /b %rc%
)

echo.
echo  ================= Bitti! =================
echo  Exe: %~dp0target\release\ffstudio.exe
echo.

if "%BUILD_ONLY%"=="1" goto :end
set /p "runit=  calistiralim mi?  e veya h: "
if /i not "%runit%"=="e" goto :end
start "" "%~dp0target\release\ffstudio.exe"

:end
pause
exit /b 0

:nocargo
echo.
echo  [HATA] cargo bulunamadi!
echo  Rust'u kur: https://rustup.rs  - varsayilan "stable MSVC" secenegini kullan.
echo  Kurulumdan sonra bu pencereyi kapatip build.bat'i tekrar calistir.
echo.
pause
exit /b 1
