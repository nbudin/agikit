echo off
set prog=kq4demo
set progid=DM
set name=The King's Quest IV Demo
set disks=1
set sg=""
set dr=%1
if "%1" == "" set dr=c:
if "%1" == "C" set dr=c:
if not exist installh.bat goto Usage
echo off
echo Hard disk installation for
echo %name%
pause

makepath %dr%\sierra
if errorlevel 1 goto CantMake
makepath %dr%%prog%
if errorlevel 1 goto CantMake
copy _install.bat %dr%
%dr%_install %dr%
goto Exit

:CantMake
echo Unable to create the directory
echo      %dr%\sierra\%prog%
echo Installation cancelled.
goto Exit

:Usage
echo off
echo To install %name% on
echo hard disk C:, make the drive
echo containing the %name%
echo disk your current drive, then type
echo      installh  C:
echo If your hard disk has a drive letter
echo other than C:, use its letter instead.

:Exit

