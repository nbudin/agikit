copy *.* %dr%\sierra\%prog%
del %dr%\sierra\%prog%\installh.bat
copy %dr%\sierra\%prog%\_%prog%.bat %dr%\sierra\%prog%.bat
del %dr%\sierra\%prog%\_%prog%.bat
ren %dr%\sierra\%prog%\sierra.com %prog%.com

%1 
cd \sierra 
echo To play %name%, type 
echo 	cd \sierra 
echo Then type 
echo 	%prog%
set dr=
set prog=
set progid=
set name=
set disks=

