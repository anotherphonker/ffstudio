@echo off
setlocal EnableExtensions
title FF Studio - GitHub'a yuckleme
rem ============================================================
rem  FF Studio - GitHub'a yuckleme scripti
rem  Hedef repo : https://github.com/anotherphonker/ffstudio
rem
rem  ILK KULLANIM (bir kere):
rem    1) Tarayicida https://github.com/new ac
rem    2) Repository name: ffstudio  -  EMPTY (README ekleme!)
rem       -  Create repository
rem    3) Bu scripti calistir.
rem
rem  SONRAKI GUNCELLEMELER: bu scripti calistirmak yeterli.
rem ============================================================

cd /d "%~dp0"

where git >nul 2>nul
if errorlevel 1 (
    echo.
    echo  [HATA] Git bulunamadi!
    echo  Kur: https://git-scm.com/downloads  -  varsayilan seceneklerle.
    echo  Kurduktan sonra bu scripti tekrar calistir.
    echo.
    pause
    exit /b 1
)

rem --- 1/4 repo hazirlama (ilk kere) ---
if not exist ".git" (
    echo.
    echo  [1/4] Yeni git repository'si hazirlaniyor...
    git init
    git branch -M main
    git remote add origin https://github.com/anotherphonker/ffstudio.git
    if errorlevel 1 goto :baglanti_hata
) else (
    echo.
    echo  [1/4] Var olan git repository'si kullaniliyor.
)

rem --- kimlik (ilk kere; sonrasinda degismez) ---
git config user.name >nul 2>nul
if errorlevel 1 git config user.name "anotherphonker"
git config user.email >nul 2>nul
if errorlevel 1 git config user.email "anotherphonker@users.noreply.github.com"

rem --- 2/4 ekleme ---
echo  [2/4] Dosyalar ekleniyor (.gitignore'a gore; target/ ve ffmpeg.zip disi)...
git add -A
git status --short

rem --- 3/4 commit ---
echo  [3/4] Commit aliniyor...
git commit -m "FF Studio guncelleme %date%"
if errorlevel 1 (
    echo.
    echo  [Bilgi] Degisiklik yok gibi gorunuyor; commit atlanabilir.
)

rem --- 4/4 itme ---
echo  [4/4] GitHub'a gonderiliyor...
echo  Ilk seferinde tarayicida GitHub giris acilabilir ya da
echo   kullanic adi + PERSONAL ACCESS TOKEN istenir:
echo   https://github.com/settings/tokens  -  "repo" izni ver.
git push -u origin main
if errorlevel 1 goto :baglanti_hata

echo.
echo  ================= Bitti! =================
echo  Repo: https://github.com/anotherphonker/ffstudio
echo.
pause
exit /b 0

:baglanti_hata
echo.
echo  [HATA] GitHub'a iletilemedi. Yukaridaki mesajlara bak:
echo   - Repo olusturuldu mu? (ilk kullanima bak)
echo   - Git kullanici adin/giris bilgilerin dogru mu?
echo   - Agirlikli (100MB+) dosya eklenmis mi? (.gitignore kontrol et)
echo.
pause
exit /b 1
