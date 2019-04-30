# migration to lovm
the new virtual machine works inherently different from the original implementation.
an essential migration problem is the fact that lovm does not have support 
for function overloading. several small modifications help for solving this:

# problem

wenn funktionen mit neuen namen übersetzt würden, könnten keine direkten vm aufrufe durchgeführt
werden. lovm hat keinen support für varargs in funktionen, da funktionen den value stack zur 
freien verfügung für eingabe und rückgabe haben.

# lösung

## varargs

das problem könnte gelöst werden, indem ein standardmäßiges parameter bei jedem funktionsaufruf mitgeben wird.
im anschluss könnte über einen generierten dispatch table die zugehörigen argument an lokale branches weitergegeben werden.

beispiel:
die funktion sqrt kennt zwei varianten: sqrt(x), sqrt(x,y).

sqrt:
 ----- begin of varargs table
 pop argc
 cmp argc, 1
 je _b1
 cmp argc, 2
 je _b2
 error?
 ret
 ----- end of varargs table
 _b1:
     pop x
     compute_push
     ret
 _b2:
     pop y
     pop x
     compute_push
     ret

## type matching

beispiel:
die funktion fib kennt drei varianten: fib(0), fib(1), fib(x).
`_vdefault` ist die variant, welche nur variablen erwartet und so
alle bedingungen erfüllt und auf jeden fall durchlaufen wird

fib:
 <<<<< begin of varargs table
 pop argc
 cmp argc, 1
 je _b1
 error?
 ret
 >>>>> end of varargs table
 _b1:
     pop x
     <<<<< begin of type table
         cmp x, 0
         je _v0
         cmp x, 1
         je _v1
         jmp _vdefault
         ret
     >>>>> end of type table
     _vdefault:
         fib(x)
         ret
     _v0:
         push 0
         ret
     _v1:
         push 0
         ret
     ret
