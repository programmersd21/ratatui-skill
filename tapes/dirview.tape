Output target/betamax/dirview.gif
Output target/betamax/dirview.png
Output target/betamax/dirview.json

Require cargo

Set Shell "bash"
Set Width 1200
Set Height 650
Set WindowBar Colorful
Set BorderRadius 8
Set TypingSpeed 20ms
Set CursorBlink false
Set WaitTimeout 30s

Hide
Type@0ms "cd \"$BETAMAX_WORKING_DIRECTORY\" && mkdir -p target/betamax"
Enter
Wait
Show

Type@0ms "./target/debug/dirview ."
Enter
Wait+Screen "items"
Screenshot target/betamax/dirview-list.png
State target/betamax/dirview-list.json
Sleep 500ms

Type "?"
Wait+Screen "Browse"
Type "jjjjjjj"
Wait+Screen "Scroll down"
Screenshot target/betamax/dirview-help.png
State target/betamax/dirview-help.json
Sleep 500ms

Esc
Wait+Screen "items"
Resize 50 16
Wait+Screen "items"
Screenshot target/betamax/dirview-narrow.png
State target/betamax/dirview-narrow.json
Resize 120 40
Wait+Screen "items"
Type "q"
Enter
Wait