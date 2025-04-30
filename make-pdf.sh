FILE="$1"

dot -Tpdf "results/tmp/$FILE.dot" -o "results/tmp/$FILE.pdf"
open -a Skim "results/tmp/$FILE.pdf"

