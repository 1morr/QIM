// probe <shared> <user> <schema> <step>...
// Each step is a librime key sequence ("hello", "{space}", "{Shift_L}{Release+Shift_L}").
// After each step prints what was committed, the input, the preedit, ascii_mode and the top candidates.
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <rime_api.h>
int main(int argc, char** argv) {
  if (argc < 5) { fprintf(stderr, "usage\n"); return 1; }
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
  for (int i = 4; i < argc; ++i) {
    api->simulate_key_sequence(s, argv[i]);
    printf("%-28s", argv[i]);
    RIME_STRUCT(RimeCommit, c);
    if (api->get_commit(s, &c)) { printf(" commit=[%s]", c.text); api->free_commit(&c); }
    const char* in = api->get_input(s);
    printf(" input=[%s] ascii=%d", in ? in : "", api->get_option(s, "ascii_mode"));
    RIME_STRUCT(RimeContext, ctx);
    if (api->get_context(s, &ctx)) {
      printf(" preedit=[%s]", ctx.composition.preedit ? ctx.composition.preedit : "");
      for (int k = 0; k < ctx.menu.num_candidates && k < 3; ++k) printf(" %d.%s", k + 1, ctx.menu.candidates[k].text);
      api->free_context(&ctx);
    }
    printf("\n");
  }
  api->destroy_session(s);
  api->finalize();
  return 0;
}
