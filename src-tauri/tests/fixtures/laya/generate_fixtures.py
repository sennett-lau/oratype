"""Regenerates the Laya reference fixtures (plan `language-router`, part `laya-head.md`).

Run once in a throwaway virtualenv; Typelite itself never runs Python:

    python3 -m venv /tmp/laya-venv
    /tmp/laya-venv/bin/pip install torch transformers safetensors tokenizers laya==0.3.20
    /tmp/laya-venv/bin/python generate_fixtures.py [--real]

Always written (small, used by `cargo test --lib`):
  tiny_head.json       a 16-wide, 2-head, 2-layer copy of Laya's head built from the same
                       PyTorch modules as `laya.common.DecisionModel`, random weights, one input
                       and its logits.
  tiny_tokenizer.json  a small tokenizer with the mmBERT settings (Replace " " -> "▁",
                       Metaspace always/split, BPE with byte fallback, added tokens).
  tiny_tokenizer_cases.json  texts and the ids the Hugging Face tokenizer gives them.

With --real (downloads convaiinnovations/laya-multilingual, about 650 MB):
  real_tokenizer_cases.json  texts and ids from the real mmBERT tokenizer.
  real_sequences.json  Laya sequences (ids, option markers, question type) and PyTorch
                       probabilities for the language question and a support-ticket example.
  billing_hidden.f32   the PyTorch backbone's last hidden states for the "billing" sequence
                       (rows x 768 little-endian f32), so the Rust head can be checked on real
                       weights without llama-server.
"""
import json
import struct
import sys

import torch
import torch.nn as nn

OUT = __file__.rsplit("/", 1)[0] if "/" in __file__ else "."


def tiny_head():
    torch.manual_seed(7)
    d, nhead, layers = 16, 2, 2
    layer = nn.TransformerEncoderLayer(d, nhead, 4 * d, 0.1, batch_first=True, norm_first=True)
    head = nn.TransformerEncoder(layer, layers, enable_nested_tensor=False)
    type_emb = nn.Embedding(3, d)
    scorer = nn.Sequential(nn.LayerNorm(d), nn.Linear(d, d), nn.GELU(), nn.Linear(d, 1))
    # Non-trivial LayerNorm parameters, so a missing scale or shift shows up.
    for mod in list(head.modules()) + list(scorer.modules()):
        if isinstance(mod, nn.LayerNorm):
            nn.init.normal_(mod.weight, 1.0, 0.2)
            nn.init.normal_(mod.bias, 0.0, 0.2)
    for mod in head.modules():
        if isinstance(mod, nn.MultiheadAttention):
            nn.init.normal_(mod.in_proj_bias, 0.0, 0.2)
    for mod in [head, type_emb, scorer]:
        mod.eval()
    rows, qtype, markers = 9, 1, [2, 4, 5]
    hidden = torch.randn(1, rows, d) * 3.0
    with torch.no_grad():
        h = hidden + type_emb(torch.tensor([qtype]))[:, None, :]
        for lyr in head.layers:
            h = lyr(h, src_key_padding_mask=torch.zeros(1, rows, dtype=torch.bool))
        logits = scorer(h[0, markers]).squeeze(-1)
    weights = {}
    for prefix, mod in [("head", head), ("type_emb", type_emb), ("scorer", scorer)]:
        for name, t in mod.state_dict().items():
            weights["%s.%s" % (prefix, name)] = {"shape": list(t.shape), "data": t.flatten().tolist()}
    fixture = {"d": d, "heads": nhead, "layers": layers, "rows": rows, "qtype": qtype,
               "markers": markers, "hidden": hidden.flatten().tolist(), "logits": logits.tolist(),
               "weights": weights}
    with open("%s/tiny_head.json" % OUT, "w") as f:
        json.dump(fixture, f)


TOKENIZER_TEXTS = [
    "Hello world", "hello  world ", " leading space", "", " ", "line\nbreak\ttab",
    "我哋今日去食飯", "價錢係$30 (HKD)", "emoji 😀!", "a <mask> b", "<start_of_turn>hi<eos>",
    "choice question: which one?", " opt: yes",
]


def tiny_tokenizer():
    from tokenizers import Tokenizer, AddedToken, models, normalizers, pre_tokenizers, trainers
    corpus = ["hello world", "hello there world", "which one is the right one", "choice question",
              "我哋今日去食飯", "今日食乜嘢", "價錢係多少", "yes no maybe", "line break tab"] * 5
    tok = Tokenizer(models.BPE(unk_token="<unk>", byte_fallback=True, fuse_unk=True))
    tok.normalizer = normalizers.Replace(" ", "▁")
    tok.pre_tokenizer = pre_tokenizers.Metaspace(replacement="▁", prepend_scheme="always", split=True)
    specials = ["<pad>", "<eos>", "<bos>", "<unk>", "<mask>"]
    trainer = trainers.BpeTrainer(vocab_size=150, special_tokens=specials, show_progress=False)
    tok.train_from_iterator(corpus, trainer)
    data = json.loads(tok.to_str())
    vocab = data["model"]["vocab"]
    for b in range(256):
        vocab["<0x%02X>" % b] = len(vocab)
    merges = data["model"]["merges"]
    final = Tokenizer(models.BPE(vocab=vocab, merges=[tuple(m) if isinstance(m, list) else tuple(m.split(" ")) for m in merges],
                                 unk_token="<unk>", byte_fallback=True, fuse_unk=True))
    final.normalizer = tok.normalizer
    final.pre_tokenizer = tok.pre_tokenizer
    final.add_special_tokens([AddedToken(t, normalized=False, lstrip=(t == "<mask>")) for t in specials])
    final.add_tokens([AddedToken("<start_of_turn>", normalized=False, special=False),
                      AddedToken("\t\t", normalized=False, special=False)])
    final.save("%s/tiny_tokenizer.json" % OUT)
    cases = [{"text": t, "ids": final.encode(t, add_special_tokens=False).ids} for t in TOKENIZER_TEXTS + ["tab\t\t\tend"]]
    with open("%s/tiny_tokenizer_cases.json" % OUT, "w") as f:
        json.dump(cases, f, ensure_ascii=False)


def real():
    import laya
    from laya.common import QTYPES, build_sequence
    agent = laya.load("convaiinnovations/laya-multilingual", device="cpu")
    tok, model, cfg = agent.tok, agent.model.float().eval(), agent.cfg
    texts = [
        "Remind me to call mom tonight.", "我明天要去北京开会，请帮我订一张机票。",
        "我明天要去北京開會，請幫我訂一張機票。", "佢哋啱啱食咗飯，你嚟唔嚟呀？",
        "我哋今個 project 嘅 deadline 係下個禮拜五。", "", " ", "  two  spaces ",
        "line one\nline two\n\ttab\t\t\tend", "emoji 😀 and 🇭🇰 flags", "Ünïcödé café naïve",
        "價錢係$30.5 (HKD)", "a <mask> b", "<start_of_turn>user hi<eos>", "𠮩𠵱 rare chars 㗎",
        "ｆｕｌｌ　ｗｉｄｔｈ", "choice question: Classify the language of the dictated text.",
    ]
    cases = [{"text": t, "ids": tok(t, add_special_tokens=False)["input_ids"]} for t in texts]
    with open("%s/real_tokenizer_cases.json" % OUT, "w") as f:
        json.dump(cases, f, ensure_ascii=False)

    lang = {"type": "choice", "instructions": "Classify the language of the dictated text.",
            "criteria": {"english": "English words", "mandarin": "Chinese characters, standard Mandarin grammar",
                         "cantonese": "Chinese characters with Cantonese words such as 嘅 咗 喺 唔 佢 哋",
                         "other": "any other language"}}
    ticket = json.dumps({"subject": "Duplicate charge on invoice 4411",
                         "body": "We were billed twice for March. Please refund the duplicate."}, ensure_ascii=False)
    items = [("billing", ticket, {"type": "choice", "instructions": "Which team should handle this?",
                                  "criteria": {"billing": "invoices, payments, refunds",
                                               "technical": "bugs and outages", "sales": "pricing"}}),
             ("churn", ticket, {"type": "noul", "instructions": "Is the customer threatening to cancel?"}),
             ("urgency", ticket, {"type": "score", "instructions": "How urgent is this?",
                                  "criteria": ["not urgent", "somewhat urgent", "very urgent"]})]
    items += [("lang", t, lang) for t in texts[:5]]
    out = []
    for name, state, q in items:
        qi = agent._to_internal(q)
        ids, markers = build_sequence(tok, state, qi, cfg["max_len"], cfg["head_max_len"])
        x = torch.tensor([ids])
        att = torch.ones_like(x)
        with torch.no_grad():
            hidden = model.encoder(input_ids=x, attention_mask=att).last_hidden_state[0]
            logits, _ = model(x, att, torch.tensor([markers]), torch.ones(1, len(markers), dtype=torch.bool),
                              torch.tensor([QTYPES[qi["t"]]]))
        crit = q.get("criteria")
        # Options as a list: JSON object key order is not kept by every reader.
        options = (list(crit.items()) if isinstance(crit, dict) else
                   [[str(i), c] for i, c in enumerate(crit)] if isinstance(crit, list) else [])
        out.append({"name": name, "state": state, "type": q["type"], "instructions": q["instructions"],
                    "options": options, "ids": ids, "markers": markers,
                    "qtype": QTYPES[qi["t"]], "logits": logits[0].tolist(),
                    "probs": torch.softmax(logits[0], -1).tolist()})
        if name == "billing":
            with open("%s/billing_hidden.f32" % OUT, "wb") as f:
                f.write(struct.pack("<%df" % hidden.numel(), *hidden.flatten().tolist()))
    with open("%s/real_sequences.json" % OUT, "w") as f:
        json.dump(out, f, ensure_ascii=False, indent=1)


if __name__ == "__main__":
    tiny_head()
    tiny_tokenizer()
    if "--real" in sys.argv:
        real()
