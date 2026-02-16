#!/usr/bin/env vhs
Output the-bird-is-the-word.gif
Set Margin 0
Set Padding 0
Set FontFamily "ZedMono NF Extd"
Set FontSize 9
Set Width 600
Set Height 750
Set Shell zsh
Hide
Type `export PATH="$PWD/target/release:$PATH"`
Enter
Type "export FLAPPY_SEED=42"
Enter
Ctrl+L
Show

Type "flappy-tui"
Enter
Sleep 2s
Type " "
Sleep 760ms
Type@620ms "     "
Sleep 2s
Type "q"
Ctrl+D
