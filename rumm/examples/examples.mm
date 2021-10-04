$[ ../../set.mm/set.mm $]

$( Additional meta-variables required for the RMM script
   Actually, only ` &Wgoal ` is really necessary and specific. $)
$v &Wgoal &W1 &C1 &C2 &C3 $.
wgoal $f wff &Wgoal $.
ww1 $f wff &W1 $.
cc1 $f class &C1 $.
cc2 $f class &C2 $.
cc3 $f class &C3 $.

$( Example theorem to be proven using Rumm $)
rummex1 $p |- ( ( ( ( ( A e. RR /\ B e. RR ) /\ C e. RR ) /\ D e. RR ) /\ D =/= 0 )
                  -> ( -u A x. ( ( B - C ) / D ) ) e. RR ) $=
  ( cr wcel wa cc0 cneg cmin cdiv simp-4l renegcld simp-4r simpllr resubcld
    wne co simplr simpr redivcld remulcld ) AEFZBEFZGZCEFZGZDEFZGZDHQZGZAIBCJ
    RZDKRUKAUCUDUFUHUJLMUKULDUKBCUCUDUFUHUJNUEUFUHUJOPUGUHUJSUIUJTUAUB $.
