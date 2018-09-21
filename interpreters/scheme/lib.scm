(define first car)
(define second cadr)
(define third caddr)
(define rest cdr)

(define quasiquote
  (macro (x)
         (define (constant? exp)
           (if (pair? exp) (eq? (car exp) 'quote) (not (symbol? exp))))
         (define (combine-skeletons left right exp)
           (cond
             ((null? right) (list 'list left))
             ((and (constant? left) (constant? right))
              (if (and (eqv? (eval left) (car exp))
                       (eqv? (eval right) (cdr exp)))
                  (list 'quote exp)
                  (list 'quote (cons (eval left) (eval right)))))
             ((and (pair? right) (eq? (car right) 'list))
              (cons 'list (cons left (cdr right))))
             (else (list 'cons left right))))
         (define (expand-quasiquote exp nesting)
           (cond
             ((vector? exp)
              (list 'apply 'vector (expand-quasiquote (vector->list exp) nesting)))
             ((null? exp) (list 'list))
             ((not (pair? exp))
              (if (constant? exp) exp (list 'quote exp)))
             ((and (eq? (car exp) 'unquote) (= (length exp) 2))
              (if (= nesting 0)
                  (second exp)
                  (combine-skeletons ''unquote
                                     (expand-quasiquote (cdr exp) (- nesting 1))
                                     exp)))
             ((and (eq? (car exp) 'quasiquote) (= (length exp) 2))
              (combine-skeletons ''quasiquote
                                 (expand-quasiquote (cdr exp) (+ nesting 1))
                                 exp))
             ((and (pair? (car exp))
                   (eq? (caar exp) 'unquote-splicing)
                   (= (length (car exp)) 2))
              (if (= nesting 0)
                  (list 'append (second (first exp))
                        (expand-quasiquote (cdr exp) nesting))
                  (combine-skeletons (expand-quasiquote (car exp) (- nesting 1))
                                     (expand-quasiquote (cdr exp) nesting)
                                     exp)))
             (else (combine-skeletons (expand-quasiquote (car exp) nesting)
                                      (expand-quasiquote (cdr exp) nesting)
                                      exp))))
         (expand-quasiquote x 0)))

; cannot use ` to define and/or/not as ` depends on and/or/not. 

(define and             
  (macro args 
         (cond ((null? args) #t)
               ((null? (rest args)) (first args))
               (else (list 'if (first args) (cons 'and (rest args)) #f)))))

(define or              
  (macro args 
         (if (null? args) 
             #f
             (cons 'cond (append (map list args) '((else #f)))))))

(define cond
  (macro exps
         (if (null? exps)
             (list 'void)
             (if (eq? 'else (caar exps))
                 (cons 'begin (cdar exps))
                 (list 
                  (list 'lambda (list 'condition)
                        (list 'if 'condition
                              (if (= 1 (length (car exps)))
                                  'condition
                                  (if (eq? '=> (cadar exps))
                                      (list (caddar exps) 'condition)
                                      (cons 'begin (cdar exps))))
                              (cons 'cond (cdr exps))))
                  (caar exps))))))

(define let 
  (macro (bindings . body) 
         (define (named-let name bindings body)
           `(let ((,name #f))
              (set! ,name (lambda ,(map first bindings) . ,body))
              (,name . ,(map second bindings))))
         (if (symbol? bindings) 
             (named-let bindings (first body) (rest body))
             `((lambda ,(map first bindings) . ,body) . ,(map second bindings)))))

(define let* 
  (macro (bindings . body)
         (if (null? bindings)
             `((lambda () . ,body))
             `(let (,(first bindings))
                (let* ,(rest bindings) . ,body)))))

(define letrec
  (macro (bindings . body)
         (let ((vars (map first bindings))
               (vals (map second bindings)))
           `(let ,(map (lambda (var) `(,var #f)) vars)
              ,@(map (lambda (var val) `(set! ,var ,val)) vars vals)
              . ,body))))

(define case
  (macro (exp . cases)
         (define (do-case case)
           (cond ((not (pair? case)) (error "bad syntax in case" case))
                 ((eq? (first case) 'else) case)
                 (else `((member? __exp__ ',(first case)) . ,(rest case)))))
         `(let ((__exp__ ,exp)) (cond . ,(map do-case cases)))))

(define do
  (macro (bindings test-and-result . body)
         (let ((variables (map first bindings))
               (inits (map second bindings))
               (steps (map (lambda (clause)
                             (if (null? (cddr clause))
                                 (first clause)   
                                 (third clause)))
                           bindings))
               (test (first test-and-result))
               (result (rest test-and-result)))
           `(letrec ((__loop__
                      (lambda ,variables
                        (if ,test
                            (begin . ,result)
                            (begin 
                              ,@body
                              (__loop__ . ,steps))))))
              (__loop__ . ,inits)))))

(define append (lambda ls
                 (define (append-two l1 l2)
                   (cond
                     ((null? l1) l2)
                     ((null? l2) l1)
                     (else (cons (car l1) (append-two (cdr l1) l2)))))
                 (if (null? ls)
                     '()
                     (append-two (car ls) (apply append (cdr ls))))))

(define (map f . l)
  (define (any-empty? l)
    (if (null? l)
        #f
        (if (null? (car l))
            #t
            (any-empty? (cdr l)))))
  (define (single-map f l)
    (if (null? l)
        '()
        (cons (f (car l)) (single-map f (cdr l)))))
  (if (any-empty? l)
      '()
      (cons (apply f (single-map car l)) 
            (apply map (cons f (single-map cdr l))))))

(define (filter f l) 
  (if (null? l)
      '()
      (let ((fst (car l))
            (rst (cdr l)))
        (if (f fst)
            (cons fst (filter f rst))
            (filter f rst)))))

(define (flat l)
  (if (null? l)
      '()
      (append (car l) (flat (cdr l)))))

(define for-each map)

(define (memq x l) 
  (cond
    ((null? l) #f)
    ((eq? (car l) x) l)
    (else (memq x (cdr l)))))

(define (memq1 x l) 
  (display l)
  (cond
    ((null? l) #f)
    ((eq? (car l) x) l)
    (else (memq1 x (cdr l)))))

(define (member? x list)
  (not (null? (member x list))))

(define (member x list)
  (if (null? list) '()                  
      (if (equal? x (car list)) list
          (member x (cdr list)))))

(define (void) (if #f 1))

(define (assq v l)
  (if (null? l)
      #f
      (if (eq? v (caar l))
          (car l)
          (assq v (cdr l)))))