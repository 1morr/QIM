// probe <shared> <user> <schema> <N> <input>...
// input "abc" -> print top N candidates; "abc#k" -> select candidate k (commit) and print commit
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <rime_api.h>
int main(int argc, char** argv) {
  if (argc < 6) { fprintf(stderr, "usage\n"); return 1; }
  RimeApi* api = rime_get_api();
  RIME_STRUCT(RimeTraits, t);
  t.shared_data_dir = argv[1]; t.user_data_dir = argv[2];
  t.distribution_name = "probe"; t.distribution_code_name = "probe"; t.distribution_version = "0";
  t.app_name = "rime.probe"; t.min_log_level = 1;
  char logdir[4096]; snprintf(logdir, sizeof logdir, "%s/../log", argv[2]); t.log_dir = logdir;
  api->setup(&t); api->initialize(&t);
  if (api->start_maintenance(True)) api->join_maintenance_thread();
  RimeSessionId s = api->create_session();
  if (!api->select_schema(s, argv[3])) { fprintf(stderr, "no schema\n"); return 2; }
  int n = atoi(argv[4]);
  for (int i = 5; i < argc; ++i) {
    char in[256]; strncpy(in, argv[i], 255); in[255]=0;
    char* hash = strchr(in, '#'); int pick = -1;
    if (hash) { *hash = 0; pick = atoi(hash+1); }
    api->clear_composition(s);
    api->simulate_key_sequence(s, in);
    RIME_STRUCT(RimeContext, ctx);
    if (pick < 0) {
      api->get_context(s, &ctx);
      printf("== %s  preedit=[%s]\n", in, ctx.composition.preedit ? ctx.composition.preedit : "");
      api->free_context(&ctx);
      RimeCandidateListIterator it = {0};
      if (api->candidate_list_begin(s, &it)) {
        while (api->candidate_list_next(&it) && it.index < n)
          printf("  %2d. %s  %s\n", it.index + 1, it.candidate.text, it.candidate.comment ? it.candidate.comment : "");
        api->candidate_list_end(&it);
      }
    } else {
      for (char* q = hash + 1; q && *q; ) { int k = atoi(q); api->select_candidate(s, k); q = strchr(q, ','); if (q) ++q; }
      if (api->get_context(s, &ctx) && ctx.composition.length > 0) api->commit_composition(s);
      api->free_context(&ctx);
      RIME_STRUCT(RimeCommit, c);
      if (api->get_commit(s, &c)) { printf(">> %s#%d commit [%s]\n", in, pick, c.text); api->free_commit(&c); }
      else printf(">> %s#%d no commit (maybe partial)\n", in, pick);
    }
  }
  api->destroy_session(s);
  api->finalize();
  return 0;
}
