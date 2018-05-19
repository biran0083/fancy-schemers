(define stream-cons (macro (a b) `(cons ,a (lambda () ,b))))
(define stream-car car)
(define stream-cdr (lambda (s) ((cdr s))))
(define stream-null? null?)

(define (stream-show s n)
  (if (and (> n 0) (not (stream-null? s)))
      (begin (display (stream-car s))
             (newline)
             (stream-show (stream-cdr s) (- n 1)))))

(define (stream-map f s)
  (if (stream-null? s)
      '()
      (stream-cons (f (stream-car s)) (stream-map f (stream-cdr s)))))

(define (stream-filter f s)
  (cond
    ((stream-null? s) '())
    ((f (stream-car s)) (stream-cons (stream-car s) (stream-filter f (stream-cdr s))))
    (else (stream-filter f (stream-cdr s)))))

(define ones (stream-cons 1 ones))

(define ints (stream-cons 1 (stream-map (lambda (x) (+ x 1)) ints)))

(define evens (stream-filter (lambda (x) (= 0 (modulo x 2))) ints))

(define (sieve s)
  (let ((fst (stream-car s)))
    (stream-cons fst
                 (sieve (stream-filter (lambda (x) (not (= 0 (modulo x fst)))) (stream-cdr s))))))

(define prime (sieve (stream-cdr ints)))

(stream-show prime 10)