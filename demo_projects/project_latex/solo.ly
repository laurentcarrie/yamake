\version "2.20.0"

songtempo=100


song_chords = \chordmode {
  g2  a2 | d2 a2 | b2:m g2 | g2 a2
  d2 g2 | g2 a2 | g1 | g1

}


lead = {
  \absolute  {
    \override Score.SpacingSpanner.shortest-duration-space = #4.0


    % les numeros de mesure ici commencent

    \set Score.currentBarNumber = 57
    % mes 1
    d8\5 d16\5 d16\5
    d16\5 d16\5 d16\5 d16\5
    d16\5 d16\5 d16\5 d16\5
    d16\5 d16\5 d8\5
    |
    % mes 2
    d8\5 e8\5 \glissando fis8\5 a8\4 b8\4 d'8\3 e'8\3 d'8\3
    |
    % mes 3
    e'4.\3 \bendAfter #+8
    e'2\3 \bendAfter #+8
    e'8\3 \bendAfter #+8 ~
    |
    % mes 4
    e'8 \3 d'8\3 a'4\2 \bendAfter #+8 a'4.\2 a'8\2 \bendAfter #+8 ~
    |
    % mes 5
    a'8\2 fis'8\2 e'16\3 \glissando fis'16\3 e'8\3 d'8\3 b8\4 d'8\3 e'8\3
    |
    % mes 6
    e'4\3 \bendAfter #+8
    e'4\3 \bendAfter #+8
    d'4.\3
    e8\5 \glissando
    |
    % mes 7
    fis8\5 a4\4 b4\4 d'4\3 e'8\3
    |
    % mes 8
    fis'8\2 a'4\2 b'4\1 d''4\1 e''8\1
    |
    % mes 8
    e''1\1 \bendAfter #+8

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
