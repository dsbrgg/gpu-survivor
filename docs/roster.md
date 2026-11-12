# Roster

## Playable founders

| Founder      | Projectile                              | Special (E, 15-second cooldown)                      |
| ------------ | --------------------------------------- | ---------------------------------------------------- |
| Jensen Huang | NVIDIA eye; one fast, hard-hitting shot | CUDA Burst: 60 area damage                           |
| Sam Altman   | OpenAI knot; two shots                  | Fundraise: 42 area damage, heal 16                   |
| Dario Amodei | Claude starburst; one heavy shot        | Guardrails: 36 area damage, 2.5-second combat shield |
| Elon Musk    | Grok ring/slash; three fast projectiles | Scale Up: 75 damage over a wider radius              |

All specials clear nearby hostile projectiles and knock rivals back. Heat ignores combat shields. Logos use pixel silhouettes; portraits and eight-direction walking poses are authored.

## Level-up weapons

The first upgrade offers Wider Context, the selected founder's signature weapon, and one random upgrade. Later choices draw from shared upgrades and that founder's weapon only. Signature weapons auto-fire alongside starter logos and stack with the E special.

| Founder | Signature         | Attack                                                                | Base interval |
| ------- | ----------------- | --------------------------------------------------------------------- | ------------- |
| Jensen  | Leather Jacket    | Piercing, spinning jackets return to the player                       | 2.2 seconds   |
| Sam     | Capital Injection | Cash bundles explode in a 55-unit radius on impact or expiry          | 2.8 seconds   |
| Dario   | Safety Rails      | Two sweeping barriers pierce rivals and intercept hostile projectiles | 3.4 seconds   |
| Elon    | Cybertruck        | Truck charges pierce rivals with strong knockback                     | 2.6 seconds   |

Signatures have three ranks. Each extra rank adds 40% base damage and one pierced rival, or ten explosion-radius units for Capital Injection. Fine-Tuning, Overclock, and stock-event damage/fire-rate modifiers also apply. Pause freezes weapon timers; replay clears upgrades.

**Wider Context** adds 20% attack radius per rank, up to four ranks. It enlarges projectile hitboxes and artwork, targeting/travel reach, E specials, Open Weights orbits, and signature areas. Pickup attraction remains Acquisition's separate upgrade. The HUD shows radius multiplier, signature rank, and its automatic firing cooldown.

## Rival attacks

| Rival               | Appearance                            | Behavior                                               |
| ------------------- | ------------------------------------- | ------------------------------------------------------ |
| On-call engineer    | Hard hat, reflective vest, wrench     | Spinning steel wrench; keeps a short throwing distance |
| Politician          | Blue suit, red tie, policy folder     | Retreats nearby; fires paired paperwork                |
| GLM                 | Violet multi-panel robot              | Three Chinese flags in a fan                           |
| Kimi                | White articulated robot, blue K visor | Fast flag shot while weaving                           |
| DeepSeek            | Blue whale with directional fins      | Paired flags followed by a charge                      |
| Lawyer              | Purple pantsuit, document case        | Subpoena with purple seal                              |
| Blue Wave inspector | Blue uniform and clipboard            | Blue inspection citation                               |
| Investor            | Burgundy waistcoat and briefcase      | Three gold coins in a spread                           |
| Maintenance bot     | Teal articulated chassis              | Short-range electrical bolt                            |
| Crawler             | Copper shell and insect legs          | Close-range scrap-nut scatter                          |
| Bernie              | White hair, navy suit                 | Twelve-document legislation ring                       |

Ordinary rivals join regular waves; Bernie belongs to the miniboss pool. Model identities reference [Z.AI's GLM documentation](https://docs.z.ai/guides/llm/glm-4.5), [Moonshot AI's Kimi repository](https://github.com/MoonshotAI/Kimi-K2), and [DeepSeek](https://api-docs.deepseek.com/). Attack patterns are fictional game mechanics, not model benchmarks.

`attack.rs` owns projectile identity, range, damage, speed, spread, and cadence. Attack icons telegraph the last 0.65 seconds; entering range cannot skip that warning. Initial cooldowns are staggered. A 128-projectile hostile budget reserves 24 slots for bosses. Dead rivals cannot fire. Pause freezes projectile animation and attack timers.

## Implemented bosses

The seed shuffles Bernie, AOC, and Alice Huang without repetition. The first arrives at 44–58 seconds. Only one miniboss is active; the next waits 18–30 seconds after defeat. Killing them grants GPU and healing drops, never victory.

- **Bernie:** twelve-document radial legislation volleys.
- **AOC:** five-subpoena fans; a visible 100-unit budget-freeze aura disables E until you leave.
- **Alice Huang — (Alleged) GPU Espionage:** cyan hacked-GPU bolts, weaving movement, periodic visual cloaking. Abilities and artwork are fictional.
- **AI Bubble:** always the final boss, arriving at the run duration (03:00). Surviving rivals and minibosses keep fighting alongside it; the boss HUD prioritizes the Bubble. Its arrival clears hostile shots. Health exceeds miniboss health by more than four times. Coin rings alternate with aimed seven-shot fans; speed and cadence intensify as health falls. Only defeating it wins.

[Al Jazeera/Reuters, 12 November 2026](https://www.aljazeera.com/news/2026/11/12/what-could-chinese-intelligence-have-learned-from-nvidia-ceos-affair) identifies Alice Huang in reporting on the espionage allegation. The report notes no publicly released evidence establishing espionage and China's denial; the game label retains “Alleged”.

## Regulator candidates — proposed encounters

The following candidates remain researched concepts. Sources establish dated policy positions, not current offices or blanket opposition to AI.

1. **Lina Khan — Antitrust Audit.** Best first mini-boss: telegraphed audit rings break up GPU clusters; beating her releases a cache. Her [25 July 2024 FTC speech](https://search.ftc.gov/system/files/ftc_gov/pdf/remarks-by-chair-lina-m-khan-at-y-combinator.pdf) discusses keeping AI markets open and competitive.
2. **Margrethe Vestager — Competition Review.** Alternating inspection lanes push players through safe gaps between racks. The [23 July 2024 joint AI competition statement](https://competition-policy.ec.europa.eu/about/news/joint-statement-competition-generative-ai-foundation-models-and-ai-products-2024-07-23_en) identifies concentrated inputs, incumbent power, and partnerships as competition concerns. Khan, Sarah Cardell, and Jonathan Kanter also signed.
3. **Richard Blumenthal + Josh Hawley — License Pending.** A paired hearing encounter: one marks permit zones, the other sends slow paperwork volleys. Their [8 September 2023 bipartisan framework](https://www.blumenthal.senate.gov/newsroom/press/release/blumenthal-and-hawley-announce-bipartisan-framework-on-artificial-intelligence-legislation) proposes independent oversight and licensing for high-risk AI development.

Keep mini-bosses optional, announce them before arrival, and cap their reinforcements within the existing enemy budget. News recipes can later schedule these encounters by supported IDs.
