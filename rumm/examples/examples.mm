$( Include set.mm $)
$[ ../../set.mm/set.mm $]

$( Declare additional meta-variables required for the RMM script.
   Scripting requires "new" variables in the sense that they should be 
   guaranteed to be distinct from any existing variable.
   metamath-knife currently does not support metavariables, 
   so these are simply manually declared. $)
$v &W1 &W2 &W3 &W4 &W5 &C1 &C2 &C3 &C4 &C5 &C6 &C7 &S1 &S2 &S3 &S4 &S5 &S6 $.
ww1 $f wff &W1 $.
ww2 $f wff &W2 $.
ww3 $f wff &W3 $.
ww4 $f wff &W4 $.
ww5 $f wff &W5 $.
cc1 $f class &C1 $.
cc2 $f class &C2 $.
cc3 $f class &C3 $.
cc4 $f class &C4 $.
cc5 $f class &C5 $.
cc6 $f class &C6 $.
cc7 $f class &C7 $.
ss1 $f setvar &S1 $.
ss2 $f setvar &S2 $.
ss3 $f setvar &S3 $.
ss4 $f setvar &S4 $.
ss5 $f setvar &S5 $.
ss6 $f setvar &S6 $.

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

${
    isleag2lem.p $e |- P = ( Base ` G ) $.
    isleag2lem.g $e |- ( ph -> G e. TarskiG ) $.
    isleag2lem.a $e |- ( ph -> A e. P ) $.
    isleag2lem.b $e |- ( ph -> B e. P ) $.
    isleag2lem.c $e |- ( ph -> C e. P ) $.
    isleag2lem.d $e |- ( ph -> D e. P ) $.
    isleag2lem.e $e |- ( ph -> E e. P ) $.
    isleag2lem.f $e |- ( ph -> F e. P ) $.
    isleag2lem.1 $e |- ( ph -> <" A B C "> ( leA ` G ) <" D E F "> ) $.
    $( Geometrical "less than" property for angles: another equivalent
       definition. Theorem 11.29 of [Schwabhauser] p. 102.  (Contributed by
       Thierry Arnoux, 17-Oct-2020.) $)
    isleag2lem $p |- ( ph -> E. y e. P ( C ( inA ` G ) <" A B y ">
        /\ <" A B y "> ( cgrA ` G ) <" D E F "> ) ) $=
      ? $.
$}

