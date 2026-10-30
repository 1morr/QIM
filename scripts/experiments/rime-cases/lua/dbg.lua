local function f(input, env)
  for cand in input:iter() do
    local g = cand:get_genuine()
    yield(ShadowCandidate(cand, cand.type, cand.text, string.format("%s [%s q=%.4f %d-%d g=%s]", cand.comment, cand.type, cand.quality, cand.start, cand._end, g.type)))
  end
end
return f
