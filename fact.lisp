(begin (define fact (lambda (n) (if (= n 0) (quote 1) (* n (fact (- n 1)))))) (fact 5))
