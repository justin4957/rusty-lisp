; Comprehensive example demonstrating all language features

; Simple calculations
(+ 1 2 3)
(* 4 5)

; String handling
"Hello, Lisp World!"

; Boolean logic  
(> 10 5)
(= 7 7)

; Conditionals
(if (< 3 5) "three is less than five" "impossible")

; Variable bindings
(let ((name "Lisp Compiler") (version 1.0))
     (list name version))

; Complex mathematical expression
(let ((a 2) (b 3) (c 4))
     (+ (* a b) (/ c 2)))

; Nested conditionals with variables
(let ((score 92))
     (if (>= score 90)
         (list "Grade" "A" "Excellent!")
         (if (>= score 80)
             (list "Grade" "B" "Good!")
             (list "Grade" "C" "Average"))))

; Area calculations
(let ((length 10) (width 5) (height 3))
     (list 
         (* length width)           ; Rectangle area
         (* 0.5 length height)      ; Triangle area  
         (* length width height)))  ; Volume