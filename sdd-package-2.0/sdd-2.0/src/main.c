/****************************************************************************************
 * The Sentential Decision Diagram Package
 * sdd version 2.0, January 8, 2018
 * http://reasoning.cs.ucla.edu/sdd
 ****************************************************************************************/

#include <time.h>
#include "sddapi.h"
#include "compiler.h"

// forward references
SddCompilerOptions sdd_getopt(int argc, char **argv); // in getopt.c
char* ppc(SddSize n); // pretty print (in libsdd: util.c)

void print_node(SddNode* node, SddManager* manager) {
  char* s = NULL;
  fprintf(stderr,    " sdd size               : %s \n",s=ppc(sdd_size(node))); free(s);
  fprintf(stderr,    " sdd node count         : %s \n",s=ppc(sdd_count(node))); free(s);

  clock_t c1, c2;
  c1 = clock();
  SddModelCount mc = sdd_global_model_count(node,manager);
  c2 = clock();
  fprintf(stderr,    " sdd model count        : %s    %.3f sec\n",s=ppc(mc),(float)(c2-c1)/CLOCKS_PER_SEC); free(s);
}

/****************************************************************************************
 * start
 ****************************************************************************************/

int main(int argc, char** argv) {

  //get options from command line (and defaults)
  SddCompilerOptions options = sdd_getopt(argc,argv);

  Fnf* fnf = NULL;
  Vtree* vtree;
  SddNode* node;
  SddManager* manager;
  clock_t c1, c2;
  char* s;

  if(options.cnf_filename!=NULL) {
    fprintf(stderr,"\nreading cnf...");
    fnf = read_cnf(options.cnf_filename);
    fprintf(stderr,"vars=%"PRIlitS" clauses=%"PRIsS"",fnf->var_count,fnf->litset_count);
  }
  else if(options.dnf_filename!=NULL) {
    fprintf(stderr,"\nreading dnf...");
    fnf = read_dnf(options.dnf_filename);
    fprintf(stderr,"vars=%"PRIlitS" terms=%"PRIsS"",fnf->var_count,fnf->litset_count);
  }

  if(options.vtree_filename!=NULL) {
    fprintf(stderr,"\nreading initial vtree...");
    vtree = sdd_vtree_read(options.vtree_filename);
  }
  else {
    fprintf(stderr,"\ncreating initial vtree (%s)...",options.initial_vtree_type);
    vtree = sdd_vtree_new(fnf->var_count,options.initial_vtree_type);
  }

  fprintf(stderr,"\ncreating manager...");
  //create manager
  manager = sdd_manager_new(vtree);
  //no longer needed
  sdd_vtree_free(vtree);
  //passing compiler options to manager
  sdd_manager_set_options(&options,manager);

  if(options.sdd_filename==NULL) {
    fprintf(stderr,"\ncompiling..."); fflush(stdout);
    c1 = clock();
    node = fnf_to_sdd(fnf,manager);
    c2 = clock();
    float secs = (float)(c2-c1)/CLOCKS_PER_SEC;
    fprintf(stderr,"\n\ncompilation time        : %.3f sec\n",secs);
    fprintf(stdout, "{ \"compilation_time\": %.3f }\n", secs);
  } else {
    fprintf(stderr,"\nreading sdd from file..."); fflush(stdout);
    c1 = clock();
    node = sdd_read(options.sdd_filename,manager);
    c2 = clock();
    float secs = (float)(c2-c1)/CLOCKS_PER_SEC;
    fprintf(stderr,"\n\nread time               : %.3f sec\n",secs);
  }

  print_node(node,manager);
  if(options.verbose)
    sdd_manager_print(manager);

  if(options.minimize_cardinality) {
    fprintf(stderr,"\nminimizing cardinality..."); fflush(stdout);
    c1 = clock();
    node = sdd_global_minimize_cardinality(node,manager);
    c2 = clock();
    SddLiteral min_card = sdd_minimum_cardinality(node);
    fprintf(stderr,"\n");
    print_node(node,manager);
    fprintf(stderr," min cardinality        : %ld   %.3f sec\n",min_card,(float)(c2-c1)/CLOCKS_PER_SEC);
  }

  Vtree* manager_vtree = sdd_manager_vtree(manager);

  if(options.post_search==1) {
    sdd_ref(node,manager);
    fprintf(stderr,"\ndynamic vtree (post compilation)\n");
    fprintf(stderr,    " sdd initial size       : %s\n",s=ppc(sdd_size(node))); free(s);
    fflush(stdout);
    c1 = clock();
    sdd_manager_minimize_limited(manager);
    c2 = clock();
    fprintf(stderr,"\n");
    fprintf(stderr,    " dynamic vtree time     : %.3f sec\n",(float)(c2-c1)/CLOCKS_PER_SEC);
    print_node(node,manager);
    sdd_deref(node,manager);
    if(options.verbose)
      sdd_manager_print(manager);
  }

  if(options.output_sdd_filename != NULL) {
    fprintf(stderr,"saving compiled sdd ...");
    sdd_save(options.output_sdd_filename,node);
    fprintf(stderr,"done\n");
  }

  if(options.output_sdd_dot_filename != NULL) {
    fprintf(stderr,"saving compiled sdd (dot)...");
    sdd_save_as_dot(options.output_sdd_dot_filename,node);
    fprintf(stderr,"done\n");
  }

  if(options.output_vtree_filename != NULL) {
    fprintf(stderr,"saving vtree...");
    sdd_vtree_save(options.output_vtree_filename,manager_vtree);
    fprintf(stderr,"done\n");
  }

  if(options.output_vtree_dot_filename != NULL) {
    fprintf(stderr,"saving vtree (dot)...");
    sdd_vtree_save_as_dot(options.output_vtree_dot_filename,manager_vtree);
    fprintf(stderr,"done\n");
  }

  fprintf(stderr,"freeing...");
  fflush(stdout);
  if(options.cnf_filename!=NULL || options.dnf_filename!=NULL) {
    free_fnf(fnf);
  }
  sdd_manager_free(manager);
  fprintf(stderr,"done\n");

  return 0;
}

/****************************************************************************************
 * end
 ****************************************************************************************/
