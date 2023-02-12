$( Include set.mm $)
$[ ../../set.mm/set.mm $]

$( Declare additional meta-variables required for the RMM script.
   Scripting requires "new" variables in the sense that they should be 
   guaranteed to be distinct from any existing variable.
   metamath-knife currently does not support metavariables, 
   so these are simply manually declared. $)
$v &W1 &W2 &W3 &W4 &C1 &C2 &C3 &C4 &C5 &C6 &C7 &S1 &S2 $.
ww1 $f wff &W1 $.
ww2 $f wff &W2 $.
ww3 $f wff &W3 $.
ww4 $f wff &W4 $.
cc1 $f class &C1 $.
cc2 $f class &C2 $.
cc3 $f class &C3 $.
cc4 $f class &C4 $.
cc5 $f class &C5 $.
cc6 $f class &C6 $.
cc7 $f class &C7 $.
ss1 $f setvar &S1 $.
ss2 $f setvar &S2 $.

${
  rummex1.1 $e |- D e. RR $.
  rummex1.2 $e |- D =/= 0 $.
  $( Example theorem to be proven using Rumm $)
  rummex1 $p |- ( ( ( A e. RR /\ B e. RR ) /\ C e. RR )
                    -> ( -u A x. ( ( B - C ) / D ) ) e. RR ) $=
    ( cr wcel wa cneg cmin cdiv simpll renegcld simplr simpr resubcld a1i cc0
      co wne redivcld remulcld ) AGHZBGHZIZCGHZIZAJBCKTZDLTUHAUDUEUGMNUHUIDUHBC
      UDUEUGOUFUGPQDGHUHERDSUAUHFRUBUC $.
$}
