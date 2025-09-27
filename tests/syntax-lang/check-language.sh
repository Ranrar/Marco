# Found at, to support syntax in sourceview5
# https://gitlab.gnome.org/GNOME/gtksourceview/-/tree/master/data/language-specs
#
#!/bin/sh
# "./check-language.sh files..." will validate files given on command line.
# "./check-language.sh" without arguments will validate all language files
# in the source directory

files=""

if [ $1 ]; then
  files=$@
else
  if [ "$srcdir" ]; then
    cd $srcdir
  fi

  for lang in *.lang; do
    case $lang in
      msil.lang) ;;
      *)
        files="$files $lang"
        ;;
    esac
  done
fi

for file in $files; do
  case $file in
  testv1.lang) ;; # skip test file for old format
  *)
    xmllint --relaxng language2.rng --noout $file || exit 1
    ;;
  esac
done
