/**************************************************************************************************
MiniSat -- Copyright (c) 2005, Niklas Sorensson
http://www.cs.chalmers.se/Cs/Research/FormalMethods/MiniSat/

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and
associated documentation files (the "Software"), to deal in the Software without restriction,
including without limitation the rights to use, copy, modify, merge, publish, distribute,
sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or
substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT
NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT
OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
**************************************************************************************************/
// Modified to compile with MS Visual Studio 6.0 by Alan Mishchenko
// Modified to implement bdd-based AllSAT solver on top of MiniSat by Takahisa Toda

#include "solver.h"

#ifdef REDUCTION
#include "bdd_interface.h"
#include "bdd_reduce.h"
#endif

#ifdef GMP
#include <gmp.h>
#endif

#include <stdio.h>
#include <stdlib.h>
#include <time.h>
//#include <unistd.h>
#include <signal.h>
//#include <zlib.h>
//#include <sys/time.h>
//#include <sys/resource.h>

//=================================================================================================
// Helpers:

#ifdef REDUCTION
DdManager *dd_mgr = NULL; //!< BDD/ZDD manager for CUDD
#endif

// Reads an input stream to end-of-file and returns the result as a 'char*' terminated by '\0'
// (dynamic allocation in case 'in' is standard input).
//
char* readFile(FILE *  in)
{
    char*   data = malloc(65536);
    int     cap  = 65536;
    int     size = 0;

    while (!feof(in)){
        if (size == cap){
            cap *= 2;
            data = realloc(data, cap); }
        size += fread(&data[size], 1, 65536, in);
    }
    data = realloc(data, size+1);
    data[size] = '\0';

    return data;
}

//static inline double cpuTime(void) {
//    struct rusage ru;
//    getrusage(RUSAGE_SELF, &ru);
//    return (double)ru.ru_utime.tv_sec + (double)ru.ru_utime.tv_usec / 1000000; }


//=================================================================================================
// DIMACS Parser:


static inline void skipWhitespace(char** in) {
    while ((**in >= 9 && **in <= 13) || **in == 32)
        (*in)++; }

static inline void skipLine(char** in) {
    for (;;){
        if (**in == 0) return;
        if (**in == '\n') { (*in)++; return; }
        (*in)++; } }

static inline int parseInt(char** in) {
    int     val = 0;
    int    _neg = 0;
    skipWhitespace(in);
    if      (**in == '-') _neg = 1, (*in)++;
    else if (**in == '+') (*in)++;
    if (**in < '0' || **in > '9') fprintf(stderr, "PARSE ERROR! Unexpected char: %c\n", **in), exit(1);
    while (**in >= '0' && **in <= '9')
        val = val*10 + (**in - '0'),
        (*in)++;
    return _neg ? -val : val; }

static void readClause(char** in, solver* s, veci* lits) {
    int parsed_lit, var;
    veci_resize(lits,0);
    for (;;){
        parsed_lit = parseInt(in);
        if (parsed_lit == 0) break;
        var = abs(parsed_lit)-1;
        veci_push(lits, (parsed_lit > 0 ? toLit(var) : lit_neg(toLit(var))));
    }
}

static lbool parse_DIMACS_main(char* in, solver* s) {
    veci lits;
    veci_new(&lits);

    for (;;){
        skipWhitespace(&in);
        if (*in == 0)
            break;
        else if (*in == 'c' || *in == 'p')
            skipLine(&in);
        else{
            lit* begin;
            readClause(&in, s, &lits);
            begin = veci_begin(&lits);
            if (!solver_addclause(s, begin, begin+veci_size(&lits))){
                veci_delete(&lits);
                return l_False;
            }
        }
    }
    veci_delete(&lits);
    return solver_simplify(s);
}


// Inserts problem into solver. Returns FALSE upon immediate conflict.
//
static lbool parse_DIMACS(FILE * in, solver* s) {
    char* text = readFile(in);
    lbool ret  = parse_DIMACS_main(text, s);
    free(text);
    return ret; }


//=================================================================================================


void printStats(stats* stats, unsigned long cpu_time, bool interrupted)
{
    double Time    = (double)(cpu_time)/(double)(CLOCKS_PER_SEC);
    fprintf(stderr,"restarts          : %12llu\n", stats->starts);
    fprintf(stderr,"conflicts         : %12.0f           (%9.0f / sec      )\n",  (double)stats->conflicts   , (double)stats->conflicts   /Time);
    fprintf(stderr,"decisions         : %12.0f           (%9.0f / sec      )\n",  (double)stats->decisions   , (double)stats->decisions   /Time);
    fprintf(stderr,"propagations      : %12.0f           (%9.0f / sec      )\n",  (double)stats->propagations, (double)stats->propagations/Time);
    fprintf(stderr,"inspects          : %12.0f           (%9.0f / sec      )\n",  (double)stats->inspects    , (double)stats->inspects    /Time);
    fprintf(stderr,"conflict literals : %12.0f           (%9.2f %% deleted  )\n", (double)stats->tot_literals, (double)(stats->max_literals - stats->tot_literals) * 100.0 / (double)stats->max_literals);
    fprintf(stderr,"cpu time (solve)  : %12.2f sec\t", Time);
    fprintf(stderr,"\n");

    fprintf(stderr,"refreshes         : %12llu\n", stats->refreshes);
    fprintf(stderr,"|obdd|            : %12llu\n", stats->obddsize);

    fprintf(stderr,"cache hits        : %12llu\n",   stats->ncachehits);
    fprintf(stderr,"cache lookup      : %12llu\n",   stats->ncachelookup);

#ifdef CUTSETCACHE
    fprintf(stderr,"cache type        : cutset\n");
#else
    fprintf(stderr,"cache type        : separator\n");
#endif

#ifdef LAZY
    fprintf(stderr,"cache frequency   : lazy\n");
#else
    fprintf(stderr,"cache frequency   : original\n");
#endif

#ifdef NONBLOCKING
    fprintf(stderr,"minisat_all type  : non-blocking\n");
#if defined(BT)
    fprintf(stderr,"backtrack method  : bt\n");
#elif defined(BJ)
    fprintf(stderr,"backtrack method  : bj\n");
#elif defined(CBJ)
    fprintf(stderr,"backtrack method  : cbj\n");
#else
    fprintf(stderr,"backtrack method  : bj+cbj\n");
#endif
#ifdef DLEVEL
    fprintf(stderr,"1UIP              : dlevel\n");
#else
    fprintf(stderr,"1UIP              : sublevel\n");
#endif
#else
    fprintf(stderr,"minisat_all type  : blocking\n");
#endif

#ifdef GMP
    fprintf(stderr,"gmp               : enabled\n");
    fprintf(stderr,"SAT (full)        : ");
    mpz_out_str(stdout, 10, stats->tot_solutions_gmp);
    if (interrupted)
        fprintf(stderr,"+");
    fprintf(stderr,"\n");
#else
    fprintf(stderr,"gmp               : disabled\n");
    fprintf(stderr,"SAT (full)        : %12ju", stats->tot_solutions);
    if (stats->tot_solutions >= INTPTR_MAX || interrupted)
        fprintf(stderr,"+");
    fprintf(stderr,"\n");
#endif
}

volatile sig_atomic_t eflag = 0;
static void SIGINT_handler(int signum)
{
	eflag = 1;
}

//=================================================================================================

static inline void PRINT_USAGE(char *p)
{
    fprintf(stderr, "Usage:\t%s [options] input-file [output-file]\n", (p));
#ifdef NONBLOCKING
#ifdef REFRESH
    fprintf(stderr, "-n<int>\tmaximum number of obdd nodes: if exceeded, obdd is refreshed\n");
#endif
#endif
}


int main(int argc, char** argv)
{
    solver* s = solver_new();
    lbool   st;
    FILE *  in;
    FILE *  out;
    s->stats.clk = clock();

    char *infile  = NULL;
    char *outfile = NULL;
    int  lim, span, maxnodes;

    /*** RECEIVE INPUTS ***/
    for (int i = 1; i < argc; i++) {
        if (argv[i][0] == '-') {
            switch (argv[i][1]) {
                case 'n':
#ifdef NONBLOCKING
#ifdef REFRESH
                    maxnodes = atoi(argv[i]+2);
                    if (maxnodes <= 0) {
                        PRINT_USAGE(argv[0]); return  0;
                    }
                    s->stats.maxnodes = maxnodes;
#endif
#endif
                    break;
                case '?': case 'h': default:
                    PRINT_USAGE(argv[0]); return  0;
            }
        } else {
            if (infile == NULL)
                infile  = argv[i];
            else if(outfile == NULL)
                outfile = argv[i];
            else
                {PRINT_USAGE(argv[0]); return  0;}
        }
    }
    if (infile == NULL)
        {PRINT_USAGE(argv[0]); return  0;}

    in = fopen(infile, "rb");
    if (in == NULL)
        fprintf(stderr, "ERROR! Could not open file: %s\n", argc == 1 ? "<stdin>" : infile), exit(1);
    if (outfile != NULL) {
        out = fopen(outfile, "wb");
        if (out == NULL)
            fprintf(stderr, "ERROR! Could not open file: %s\n", argc == 1 ? "<stdin>" : outfile), exit(1);
#ifdef NONBLOCKING
        else s->out = out;
#endif
    } else {
        out = NULL;
    }


    st = parse_DIMACS(in, s);
    fclose(in);

    if (st == l_False){
        solver_delete(s);
        fprintf(stderr,"Trivial problem\nUNSATISFIABLE\n");
        exit(20);
    }

    s->verbosity = 0; // changed by are_we_sdd_yet
    if (signal(SIGINT, SIGINT_handler) == SIG_ERR) {
        fprintf(stderr, "ERROR! Cound not set signal");
        exit(1);
    }

    st = solver_solve(s,0,0);

    fprintf(stderr,"input             : %s\n", infile);
    fprintf(stderr,"variables         : %12d\n",   s->size);
#ifdef CUTSETCACHE
    fprintf(stderr,"cutwidth          : %12d\n",   s->maxcutwidth);
#else
    fprintf(stderr,"pathwidth         : %12d\n",   s->maxpathwidth);
#endif
	if (eflag == 1) {
    	fprintf(stderr,"\n"); fprintf(stderr,"*** INTERRUPTED ***\n");
    	printStats(&s->stats, clock() - s->stats.clk, true);
    	fprintf(stderr,"\n"); fprintf(stderr,"*** INTERRUPTED ***\n");
	} else {
    	printStats(&s->stats, clock() - s->stats.clk, false);
	}

    // addition by are-we-sdd-yet
    double Time    = (double)(clock() - s->stats.clk)/(double)(CLOCKS_PER_SEC);
    fprintf(stdout, "{\"time\": %.8f }", Time);

    if (outfile != NULL)
        obdd_decompose(out, s->size, s->root);

#ifdef REDUCTION
    if (s->stats.refreshes == 0) { // perform reduction if obdd has not been refreshed.
        bdd_init(s->size,0);
        clock_t starttime_reduce = clock();
        bddp  f = bdd_reduce(s->root);
        clock_t endtime_reduce = clock();
        fprintf(stderr,"cpu time (reduce) : %12.2f sec\n", (float)(endtime_reduce - starttime_reduce)/(float)(CLOCKS_PER_SEC));
        fprintf(stderr,"|bdd|             : %12ju\n",  bdd_size(f));
    }
#endif

    solver_delete(s);
    return 0;
}
