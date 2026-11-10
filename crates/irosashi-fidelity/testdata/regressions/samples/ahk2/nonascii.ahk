; Hotkeys whose key names are outside ASCII, one per line.
a::Send "ascii"
é::Send "latin-1"
ÿ::Send "latin-1 high"
Ā::Send "latin extended"
中::Send "cjk"
😀::Send "astral"
ſ::Send "long s, folds to s"
K::Send "kelvin sign, folds to k"
^中::Send "modifier"
中 & ж::Send "combination"
Ā up::Send "key up"
中:: {
    MsgBox "block body"
}
