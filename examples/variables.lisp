; Variable bindings with let
(let ((x 10) (y 20))
     (+ x y))

; Nested let bindings
(let ((a 5))
     (let ((b (* a 2)))
          (+ a b)))

; Multiple calculations with variables
(let ((base 10) (height 15) (width 8))
     (* base height width))

; Mathematical formulas
(let ((pi 3.14159) (radius 5))
     (* pi (* radius radius)))