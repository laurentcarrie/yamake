\version "2.23.1"

songtempo=80

% Bb
ma = {

}

mg = {
  <g,\6 d\5 g\4>8
  <g,\6 d\5 g\4>8
  <g,\6 d\5 g\4>8
  r8
  <g,\6 d\5 g\4>8
  <g,\6 d\5 g\4>8
  <g,\6 d\5 g\4>8
  r8
}
mc = {
  <c\5 g\4 c'\3>8
  <c\5 g\4 c'\3>8
  <c\5 g\4 c'\3>8
  r8

  <c\5 g\4 c'\3>8
  <c\5 g\4 c'\3>8
  <c\5 g\4 c'\3>8
  r8
}
mf = {
  <f,\6 c\5 f\4>8
  <f,\6 c\5 f\4>8
  <f,\6 c\5 f\4>8
  r8
  <f,\6 c\5 f\4>8
  <f,\6 c\5 f\4>8
  <f,\6 c\5 f\4>8
  r8
}

lignea = {
  \mg | \mg | \mc | \mg |
}

ligneb = {
  \mc | \mc | \mf | \mf |
}

song_chords = \chordmode {
  g1:5
  |
  g1:5
  |
  g1:5 | g1:5 | c1:5 | g1:5 |
  g1:5 | g1:5 | c1:5 | g1:5 |
  g1:5 | g1:5 | c1:5 | g1:5 |
  g1:5 | g1:5 | c1:5 | g1:5 |
  c1:5 | c1:5 | f1:5 | f1:5 |

}


lead = {
  \absolute  {
    \override Score.SpacingSpanner.shortest-duration-space = #4.0


    % les numeros de mesure ici commencent

    %
    r1
    |
    % ( levée )
    r4 r4 r4 d'8\4 f'8\3
    |


    \set Score.currentBarNumber = 1
    % mes 1
    g'2\3 r4 d''8\2\^ c''8\2
    |

    % mes 2
    %\grace c'8\2 \preBendHold d'8\2 \bendHold \^ g'8~\3 \^ g'8\3 ais'8\2 r8 r4 d'8\4
    bes'8\2  g'8~\3 g'4\3 c''8\2 r8 r8 d'8\4
    |

    % mes 3
    %   f'8\3 g'8\3 bes'8\2 \mypull bes'16\2 c''8~\2 c''4\2 bes'8\2 a'8\2
    f'8\3 g'8\3 bes'8\2  \grace bes'8\2 \^ c''8\2 c''4\2 \bendHold \^ bes'8\2  a'8\2
    |

    % mes 4
    g'8\2 a'8\2 \times 2/3 { g'16\2 a'16\2 g'16\2 } f'8\3 g'8\2 f'8\3 dis'8\3 f'8\3
    |

    % mes 5
    %   f'8\3 g'8\2 fis'8\3 g'8~\2 \mypull f'16\3 g'4\3 r8 g'8\2
    f'8\3 g'8\2 \grace f'8\3 \^ g'8\2 g'8~\2 g'4\2 r8 g'8\2
    |

    % mes 6
    bes'8\2 \mypull bes'16\2 c''8\2 bes'8\2 \mypull bes'16\2 c''8~\2 c''8\2 c''8\1 g'8\2 bes'8\2
    |

    % mes 7
    \mypull fis'32\3 g'16\3 g'16\2 c''16\1 bes'16\2
    a'16\2 g'16\2 ges'16\3  f'16\3
    ees'16\3 c'16\4 f'16\3 ees'16\3
    d'16\3 c'16\4 bes16\4 a16\4
    |

    % mes 8
    g8\4 bes8\4 a8\4 c'8\3 bes8\4 ees'8\3 d'8\3 c'8\3
    |

    % mes 9
    \mypull bes8\3 c'4~\3 c'4\3 c'8\3 bes8\3 a8\3 bes8\3

    % mes 10
    a8\3 bes8\3 a8\3 g8~\3 g2\3

    % mes 11
    \mypull c'16\3 d'16\3 d'16\2 g'16\1  \mypull f'16\2 g'16\2
    g'\1 g'\1 f'\2 e'\2
    d'\2 des'\3 c'\3 bes\3
    \times 2/3 { g8\4 c'8\3 bes8\3 }
    |

    % mes 12
    \times 2/3 { \mypull c'16\3 d'4\3 \mypull c'16\3 d'4\3 \mypull c'16\3 d'4\3 }
    \mypull c'16\3 d'4\3
    d'8\3 f'8\2
    |

    % mes 13
    g'8\2 d'8\3 f'8\2 g'8\2  bes'8\1 \mypull bis'16\1 c''8~ \mypulled  c''4
    |

    % mes 14
    \mypull c''16\1 c''8\1 c''8\1 bes'8\1 c''8\1
    g'4~\2 g'8\2 g'8\2
    |

    % mes 15
    bes'8\2 \mypull fis'8\3 g'16\3 g'16\2
    c''16\1 bes'16\2 g'16\2 c''16\1
    bes'16\2 g'16\2 c''16\1 bes'16\2
    c''16\1 r16 d''16\1 r16
    |

    % mes 16
    ees''8\1 f''8\1 r8
    \mypull f''16\1 g''8~\1 g''4\1 r8 \mypull f''16\1 g''8~\1
    |

    % mes 17
    g''8\1 f''8\1 \mypull f''16\1 g''8\1  g''8\1
    \mypull f''16\1 g''8~\1 g''4\1 f''8\1
    |

    % mes 18
    \mypull f''16\1 g''8\1 f''8\1 d''8\1 c''8\2
    \mypull c''16\2 d''8~\2 d''4\2 r8
    |

    % mes 19
    c'16\4 d'16\4 f'16\3 r16
    \myrelease d'16\4 c'16\4   bes16\4 bis16\4 r16
    c'8\4 bes8\4 g4\5
    |

    % mes 20
    f'16\3 g'16\3 c''16\2 r16
    bes'8\2 c''8\2
    \mypull c''16\2 d''2\2
    |

    % mes 21
    \mypull c''16\2 d''8\2
    \mypull c''16\2 d''8\2
    \mypull c''16\2 d''8\2
    \mypull c''16\2 d''8\2
    \mypull c''16\2 d''8\2
    \mypull c''16\2 d''8\2
    \mypull c''16\2 d''8\2
    \mypull c''16\2 d''8\2
    |

    % mes 22
    \times 2/3 {
      \mypull c''16\2 d''4\2
      \mypull c''16\2 d''4\2
      \mypull c''16\2 d''4\2
    }
    \times 2/3 {
      \mypull c''16\2 d''4\2
      \mypull c''16\2 d''4\2
      f'4\3
    }
    |

    % mes 23
    g'2\3 r2


  }

}




\paper {
  #(include-special-characters)
  indent = 0\mm
  line-width = 180\mm
  oddHeaderMarkup = ""
  evenHeaderMarkup = ""
  oddFooterMarkup = ""
  evenFooterMarkup = ""

  #(add-text-replacements!
    '(
       ("100" . "hundred")
       ("dpi" . "dots per inch")
       ))

}


\score {
  <<
    \new ChordNames {
      \song_chords
    }

    \new TabStaff {
      \tempo 4 = \songtempo
      \tabFullNotation
      \override Score.BarNumber.break-visibility = ##(#t #t #t)
      \lead
    }

  >>

  \layout {}
}
