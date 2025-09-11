; Basic conditionals
(if (> 5 3) "five is greater" "five is not greater")
(if (< 2 1) "impossible" "as expected")

; Nested conditionals for grade assignment
(let ((score 85))
     (if (>= score 90)
         "A"
         (if (>= score 80)
             "B"
             (if (>= score 70)
                 "C"
                 "F"))))

; Complex conditions
(if (and (> 10 5) (< 3 7))
    "both conditions true"
    "at least one false")