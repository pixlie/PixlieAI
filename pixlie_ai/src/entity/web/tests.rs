// Copyright 2025 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/pixlie/PixlieAI/blob/main/LICENSE

#[test]
fn test_webpage_scraper_rlhf_book() {
    use crate::engine::engine::get_test_engine;
    use crate::engine::node::{ArcedNodeItem, NodeLabel, Payload};
    use crate::engine::EdgeLabel;
    use crate::entity::web::link::Link;
    use std::sync::Arc;
    use url::Url;

    let test_engine = get_test_engine();
    let arced_test_engine = Arc::new(&test_engine);
    let link_node_id = Link::add(
        arced_test_engine,
        &"https://rlhfbook.com/c/01-introduction.html".to_string(),
        vec![NodeLabel::AddedByUser, NodeLabel::Link],
        vec![],
        true,
    )
    .unwrap();

    let webpage = RLHF_BOOK_INTRO.to_string();
    let webpage_node_id = test_engine
        .get_or_add_node(
            Payload::Text(webpage),
            vec![NodeLabel::Content, NodeLabel::WebPage],
            true,
            None,
        )
        .unwrap()
        .get_node_id();
    test_engine
        .add_connection(
            (link_node_id, webpage_node_id.clone()),
            (EdgeLabel::PathOf, EdgeLabel::ContentOf),
        )
        .unwrap();
    test_engine.process_nodes();

    let parent_of_webpage = test_engine
        .get_node_ids_connected_with_label(&webpage_node_id, &EdgeLabel::ContentOf)
        .unwrap();
    assert_eq!(parent_of_webpage.len(), 1);

    let children_of_webpage = test_engine
        .get_node_ids_connected_with_label(&webpage_node_id, &EdgeLabel::ParentOf)
        .unwrap();
    assert_eq!(children_of_webpage.len(), 86);

    let title_node = test_engine
        .get_node_by_id(children_of_webpage.first().unwrap())
        .unwrap();
    assert_eq!(
        match title_node.payload {
            Payload::Text(ref text) => text.as_str(),
            _ => "",
        },
        "Introduction | RLHF Book by Nathan Lambert"
    );
    assert_eq!(
        title_node.labels,
        vec![NodeLabel::Title, NodeLabel::Partial]
    );

    let heading_node = test_engine
        .get_node_by_id(children_of_webpage.get(1).unwrap())
        .unwrap();
    assert_eq!(
        match heading_node.payload {
            Payload::Text(ref text) => text.as_str(),
            _ => "",
        },
        "A Little Bit of Reinforcement Learning from Human Feedback"
    );

    let mut paragraph_nodes = test_engine.get_node_ids_with_label(&NodeLabel::Paragraph);
    paragraph_nodes.sort();
    assert_eq!(paragraph_nodes.len(), 37);

    let paragraph = test_engine
        .get_node_by_id(paragraph_nodes.get(2).unwrap())
        .unwrap();
    assert_eq!(
        match paragraph.payload {
            Payload::Text(ref text) => text.as_str(),
            _ => "",
        },
        "Reinforcement learning from Human Feedback (RLHF) is a technique used to incorporate human information into AI systems. RLHF emerged primarily as a method to solve hard to specify problems. Its early applications were often in control problems and other traditional domains for reinforcement learning (RL). RLHF became most known through the release of ChatGPT and the subsequent rapid development of large language models (LLMs) and other foundation models."
    );

    let paragraph = test_engine
        .get_node_by_id(paragraph_nodes.get(4).unwrap())
        .unwrap();
    assert_eq!(
        match paragraph.payload {
            Payload::Text(ref text) => text.as_str(),
            _ => "",
        },
        "RLHF has been applied to many domains successfully, with complexity increasing as the techniques have matured. Early breakthrough experiments with RLHF were applied to deep reinforcement learning [1], summarization [2], following instructions [3], parsing web information for question answering [4], and “alignment” [5]."
    );

    let link_node_ids = test_engine.get_node_ids_with_label(&NodeLabel::Link);
    assert_eq!(link_node_ids.len(), 15);

    let domain_node_ids = test_engine.get_node_ids_with_label(&NodeLabel::Domain);
    assert_eq!(domain_node_ids.len(), 8);

    let all_domain_nodes: Vec<ArcedNodeItem> = domain_node_ids
        .iter()
        .filter_map(|node_id| match test_engine.get_node_by_id(node_id) {
            Some(node) => {
                if node.labels.contains(&NodeLabel::Domain) {
                    match node.payload {
                        Payload::Text(_) => Some(node.clone()),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            None => None,
        })
        .collect();

    let some_links = [
        "https://huggingface.co/allenai/tulu-2-dpo-70b",
        "https://huggingface.co/allenai/OLMo-7B-Instruct",
        "https://huggingface.co/allenai/tulu-2-dpo-70b",
        "https://github.com/huggingface/trl",
    ];
    // Check that all the links in some_links are found in all_anchor_nodes
    let mut count_matches = 0;
    for some_link in some_links {
        match Url::parse(some_link) {
            Ok(some_link_url) => {
                let some_domain = some_link_url.domain().unwrap();
                let some_path = some_link_url.path();
                let some_query = some_link_url.query();

                // Check this domain exists as a node
                for domain_node in all_domain_nodes.iter() {
                    match &domain_node.payload {
                        Payload::Text(domain) => {
                            if some_domain == domain {
                                // Get the link node for this domain
                                let link_node_ids = test_engine
                                    .get_node_ids_connected_with_label(
                                        &domain_node.id,
                                        &EdgeLabel::OwnerOf,
                                    )
                                    .unwrap();
                                let link_nodes = link_node_ids
                                    .iter()
                                    .filter_map(|node_id| {
                                        match test_engine.get_node_by_id(node_id) {
                                            Some(node) => match node.payload {
                                                Payload::Link(_) => Some(node.clone()),
                                                _ => None,
                                            },
                                            None => None,
                                        }
                                    })
                                    .collect::<Vec<ArcedNodeItem>>();
                                for link_node in link_nodes {
                                    match &link_node.payload {
                                        Payload::Link(link) => {
                                            if some_path == link.path
                                                && some_query == link.query.as_deref()
                                            {
                                                count_matches += 1;
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            Err(err) => {
                panic!("Error parsing URL: {}", err);
            }
        }
    }
    assert_eq!(count_matches, some_links.len());

    let mut bullet_points_node_ids =
        test_engine.get_node_ids_with_label(&NodeLabel::UnorderedPoints);
    bullet_points_node_ids.sort();
    assert_eq!(bullet_points_node_ids.len(), 7);

    let first_bullet_point_node = test_engine
        .get_node_by_id(bullet_points_node_ids.first().unwrap())
        .unwrap();
    assert_eq!(
        first_bullet_point_node.labels,
        vec![NodeLabel::UnorderedPoints, NodeLabel::Partial,]
    );

    let list_item_node_ids = test_engine
        .get_node_ids_connected_with_label(&first_bullet_point_node.id, &EdgeLabel::ParentOf)
        .unwrap();
    assert_eq!(list_item_node_ids.len(), 2);
    let list_item_nodes = list_item_node_ids
        .iter()
        .map(|node_id| test_engine.get_node_by_id(node_id).unwrap())
        .collect::<Vec<ArcedNodeItem>>();
    assert_eq!(
        list_item_nodes
            .iter()
            .map(|x| match x.payload {
                Payload::Text(ref text) => text.as_str(),
                Payload::Tree => "_tree_",
                _ => "",
            })
            .collect::<Vec<_>>(),
        vec!["Introduction", "Bibliography"]
    );

    let second_bullet_point_node = test_engine
        .get_node_by_id(bullet_points_node_ids.get(1).unwrap())
        .unwrap();
    assert_eq!(
        second_bullet_point_node.labels,
        vec![NodeLabel::UnorderedPoints, NodeLabel::Partial,]
    );

    let list_item_node_ids = test_engine
        .get_node_ids_connected_with_label(&second_bullet_point_node.id, &EdgeLabel::ParentOf)
        .unwrap();
    assert_eq!(list_item_node_ids.len(), 4);
    let list_item_nodes = list_item_node_ids
        .iter()
        .map(|node_id| test_engine.get_node_by_id(node_id).unwrap())
        .collect::<Vec<ArcedNodeItem>>();
    assert_eq!(
        list_item_nodes
            .iter()
            .map(|x| match x.payload {
                Payload::Text(ref text) => text.as_str(),
                Payload::Tree => "_tree_",
                _ => "",
            })
            .collect::<Vec<_>>(),
        vec![
            "What Does RLHF Do?",
            "How We Got Here",
            "Scope of This Book",
            "Future of RLHF"
        ]
    );

    let third_bullet_point_node = test_engine
        .get_node_by_id(bullet_points_node_ids.get(2).unwrap())
        .unwrap();
    assert_eq!(
        third_bullet_point_node.labels,
        vec![NodeLabel::UnorderedPoints, NodeLabel::Partial,]
    );

    let list_item_node_ids = test_engine
        .get_node_ids_connected_with_label(&third_bullet_point_node.id, &EdgeLabel::ParentOf)
        .unwrap();
    assert_eq!(list_item_node_ids.len(), 4);
    let list_item_nodes = list_item_node_ids
        .iter()
        .map(|node_id| test_engine.get_node_by_id(node_id).unwrap())
        .collect::<Vec<ArcedNodeItem>>();
    assert_eq!(
        list_item_nodes
            .iter()
            .map(|x| match x.payload {
                Payload::Text(ref text) => text.as_str(),
                Payload::Tree => "_tree_",
                _ => "",
            })
            .collect::<Vec<_>>(),
        vec![
            "Chapter Summaries",
            "Target Audience",
            "How to Use This Book",
            "About the Author"
        ]
    );
}

const RLHF_BOOK_INTRO: &str = r###"
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" lang="en-US" xml:lang="en-US">

<head>
    <meta charset="utf-8" />
    <meta name="generator" content="pandoc" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0, user-scalable=yes" />
    <link rel="shortcut icon" type="image/x-icon" href="favicon.ico">

    <!-- Add Open Graph meta tags for share image -->
    <meta property="og:image" content="https://github.com/natolambert/rlhf-book/blob/main/images/rlhf-book-share" />
    <meta property="og:image:width" content="1920" />
    <meta property="og:image:height" content="1080" />

    <!-- <meta property="og:title" content="A Little Bit of Reinforcement Learning from Human Feedback" /> -->
    <meta property="og:title" content="Introduction | RLHF Book by Nathan Lambert" />
    <meta property="og:description" content="The Reinforcement Learning from Human Feedback Book" />
    <meta property="og:url" content="https://rlhfbook.com" />

    <meta name="author" content="Nathan Lambert" />
    <meta name="dcterms.date" content="2025-02-26" />
    <!-- <title>RLHF Book</title> -->
    <!-- SEO and Open Graph titles -->
    <title>
        Introduction | RLHF Book by Nathan Lambert
    </title>

</head>

<body>
    <header id="title-block-header">
        <h1 class="title"><a href="https://rlhfbook.com/" style="color: inherit; text-decoration: none;">A
Little Bit of Reinforcement Learning from Human Feedback</a></h1>
        <p class="subtitle">A short introduction to RLHF and post-training focused on language models.</p>

        <p class="author">Nathan Lambert</p>

        <navigation-dropdown expanded="false"></navigation-dropdown>

    </header>
    <div>
        <h3> Chapter Contents </h3>
    </div>
    <nav id="TOC" role="doc-toc">
        <ul>
            <li><a href="#introduction" id="toc-introduction">Introduction</a>
                <ul>
                    <li><a href="#what-does-rlhf-do" id="toc-what-does-rlhf-do">What Does
RLHF Do?</a></li>
                    <li><a href="#how-we-got-here" id="toc-how-we-got-here">How We Got
Here</a></li>
                    <li><a href="#scope-of-this-book" id="toc-scope-of-this-book">Scope of
This Book</a>
                        <ul>
                            <li><a href="#chapter-summaries" id="toc-chapter-summaries">Chapter
Summaries</a></li>
                            <li><a href="#target-audience" id="toc-target-audience">Target
Audience</a></li>
                            <li><a href="#how-to-use-this-book" id="toc-how-to-use-this-book">How to
Use This Book</a></li>
                            <li><a href="#about-the-author" id="toc-about-the-author">About the
Author</a></li>
                        </ul>
                    </li>
                    <li><a href="#future-of-rlhf" id="toc-future-of-rlhf">Future of
RLHF</a></li>
                </ul>
            </li>
            <li><a href="#bibliography" id="toc-bibliography">Bibliography</a></li>
        </ul>
    </nav>
    <div id="content">
        <h1 id="introduction">Introduction</h1>
        <p>Reinforcement learning from Human Feedback (RLHF) is a technique used to incorporate human information into AI systems. RLHF emerged primarily as a method to solve hard to specify problems. Its early applications were often in control problems and other traditional domains for reinforcement learning (RL). RLHF became most known through the release of ChatGPT and the subsequent rapid development of large language models (LLMs) and other foundation models.</p>
        <p>The basic pipeline for RLHF involves three steps. First, a language model that can follow user questions must be trained (see Chapter 9). Second, human preference data must be collected for the training of a reward model of human preferences (see Chapter 7). Finally, the language model can be optimized with a RL optimizer of choice, by sampling generations and rating them with respect to the reward model (see Chapter 3 and 11). This book details key decisions and basic implementation examples for each step in this process.</p>
        <p>RLHF has been applied to many domains successfully, with complexity increasing as the techniques have matured. Early breakthrough experiments with RLHF were applied to deep reinforcement learning
            <span class="citation" data-cites="christiano2017deep"><a
  href="#ref-christiano2017deep" role="doc-biblioref">[1]</a></span>, summarization <span class="citation" data-cites="stiennon2020learning"><a href="#ref-stiennon2020learning"
  role="doc-biblioref">[2]</a></span>, following instructions <span class="citation" data-cites="ouyang2022training"><a
  href="#ref-ouyang2022training" role="doc-biblioref">[3]</a></span>, parsing web information for question answering <span class="citation" data-cites="nakano2021webgpt"><a href="#ref-nakano2021webgpt"
  role="doc-biblioref">[4]</a></span>, and “alignment” <span class="citation" data-cites="bai2022training"><a
  href="#ref-bai2022training" role="doc-biblioref">[5]</a></span>.</p>
        <p>In modern language model training, RLHF is one component of post-training. Post-training is a more complete set of techniques and best-practices to make language models more useful for downstream tasks <span class="citation" data-cites="lambert2024t"><a
  href="#ref-lambert2024t" role="doc-biblioref">[6]</a></span>. Post-training can be summarized as using three optimization methods:
        </p>
        <ol type="1">
            <li>Instruction / Supervised Finetuning (IFT/SFT), where we teach formatting and for base of instruction following abilities. This is largely about learning <em>features</em> in language.</li>
            <li>Preference Finetuning (PreFT),where we align to human preferences (and get smaller bump in capabilities at the same time). This is largely about <em>style</em> of language and subtle human preferences that are hard to quantify.</li>
            <li>Reinforcement Finetuning (RFT). The newest type of post-training that boosts performance on verifiable domains.</li>
        </ol>
        <p>This book focuses on the second area, <strong>preference
  finetuning</strong>, which has more complexity than instruction tuning and is far more established than Reinforcement Finetuning. That being said, RLHF colloquially <em>is</em> what led to modern post-training. Soon after the release of ChatGPT, RLHF encompassed all of post-training. The foundations of RLHF involve far more than preferences alone and this book provides introductions to all the related topics.</p>
        <h2 id="what-does-rlhf-do">What Does RLHF Do?</h2>
        <p>The biggest question around RLHF, yet one that is still hard to answer, is “What does RLHF training offer models?” The core role of this book, beyond teaching the techniques for doing RLHF, is to distill intuition as to <em>why</em> RLHF is crucial to modern AI models. In recent years, language models shifted from academic experiments studied in the purview of benchmarks to general purpose technology. RLHF is at the core of this transition.</p>
        <p>The most compelling view of how RLHF works is to think of how
            <em>style</em> applies to interactions you have with language models. The style, or format, of information presented is crucial to how it is learned. This has always been the case for examples such as coursework, but is normally applied in the background and not considered directly.</p>
        <p>Modern research has established RLHF as a general method to integrate subtle stylistic and related behavioral features into the models. Compared to other techniques for post-training, such as instruction finetuning, RLHF generalizes far better across domains
            <span class="citation" data-cites="kirk2023understanding"><a
  href="#ref-kirk2023understanding" role="doc-biblioref">[7]</a></span>
            <span class="citation" data-cites="chu2025sft"><a
  href="#ref-chu2025sft" role="doc-biblioref">[8]</a></span> – helping create effective general purpose models.</p>
        <p>Intuitively, this can be seen in how the optimization techniques are applied. Instruction finetuning is training the model to predict the next certain token when the text preceding is close to examples it has seen. It is optimizing the model to more regularly output specific features in text. This is a per-token update.</p>
        <p>RLHF on the other hand tunes the responses on the response level rather than looking at the next token specifically. Additionally, it is telling the model what a <em>better</em> response looks like, rather than a specific response it should learn. RLHF also shows a model which type of response it should avoid, i.e. negative feedback. The training to achieve this is often called a <em>contrastive</em> loss function and is referenced throughout this book.</p>
        <p>While this flexibility is a major advantage of RLHF, it comes with implementation challenges. Largely, these center on <em>how to control
  the optimization.</em> As we will cover in this book, implementing RLHF often requires training a reward model, of which best practices are not strongly established and depend on the area of application. With this, the optimization itself is prone to
            <em>over-optimization</em> because our reward signal is at best a proxy objective, requiring regularization. With these limitations, effective RLHF requires a strong starting point, so RLHF cannot be a solution to every problem alone and needs to be approached in a broader lens of post-training.</p>
        <p>Due to this complexity, implementing RLHF is far more costly than simple instruction finetuning and can come with unexpected challenges such as length bias <span class="citation" data-cites="singhal2023long"><a href="#ref-singhal2023long"
  role="doc-biblioref">[9]</a></span> <span class="citation" data-cites="park2024disentangling"><a
  href="#ref-park2024disentangling"
  role="doc-biblioref">[10]</a></span>. For projects where performance matters, RLHF is established as being crucial to achieving a strong finetuned model, but it is more expensive in compute, data costs, and time.
        </p>
        <h2 id="how-we-got-here">How We Got Here</h2>
        <p>Why does this book make sense now? How much still will change?</p>
        <p>Post-training, the craft of eliciting powerful behaviors from a raw pretrained language model, has gone through many seasons and moods since the release of ChatGPT that sparked the renewed interest in RLHF. In the era of Alpaca <span class="citation" data-cites="alpaca"><a href="#ref-alpaca"
  role="doc-biblioref">[11]</a></span>, Vicuna <span class="citation" data-cites="vicuna2023"><a href="#ref-vicuna2023"
  role="doc-biblioref">[12]</a></span>, <span class="citation" data-cites="koala_blogpost_2023"><a href="#ref-koala_blogpost_2023"
  role="doc-biblioref">[13]</a></span>, and Dolly <span class="citation" data-cites="DatabricksBlog2023DollyV1"><a
  href="#ref-DatabricksBlog2023DollyV1"
  role="doc-biblioref">[14]</a></span>, a limited number of human datapoints with extended synthetic data in the style of Self-Instruct were used to normally fine-tune the original LLaMA to get similar behavior to ChatGPT. The benchmark for these early models was fully vibes (and human evaluation) as we were all so captivated by the fact that these small models can have such impressive behaviors across domains. It was justified excitement.</p>
        <p>Open post-training was moving faster, releasing more models, and making more noise than its closed counterparts. Companies were scrambling, e.g. DeepMind merging with Google or being started, and taking time to follow it up. There are phases of open recipes surging and then lagging behind.</p>
        <p>The era following Alpaca et al., the first lag in open recipes, was one defined by skepticism and doubt on reinforcement learning from human feedback (RLHF), the technique OpenAI highlighted as crucial to the success of the first ChatGPT. Many companies doubted that they needed to do RLHF. A common phrase – “instruction tuning is enough for alignment” – was so popular then that it still holds heavy weight today despite heavy obvious pressures against it.</p>
        <p>This doubt of RLHF lasted, especially in the open where groups cannot afford data budgets on the order of $100K to $1M. The companies that embraced it early ended up winning out. Anthropic published extensive research on RLHF through 2022 and is now argued to have the best post-training <span class="citation" data-cites="askell2021general"><a href="#ref-askell2021general"
  role="doc-biblioref">[15]</a></span> <span class="citation" data-cites="bai2022training"><a href="#ref-bai2022training"
  role="doc-biblioref">[5]</a></span> <span class="citation" data-cites="bai2022constitutional"><a
  href="#ref-bai2022constitutional"
  role="doc-biblioref">[16]</a></span>. The delta between open groups, struggling to reproduce, or even knowing basic closed techniques, is a common theme.</p>
        <p>The first shift in open alignment methods and post-training was the story of Direct Preference Optimization (DPO) <span class="citation" data-cites="rafailov2024direct"><a href="#ref-rafailov2024direct"
  role="doc-biblioref">[17]</a></span>. The DPO paper, posted in May of 2023, didn’t have any clearly impactful models trained with it going through the fall of 2023. This changed with the releases of a few breakthrough DPO models – all contingent on finding a better, lower, learning rate. Zephyr-Beta <span class="citation" data-cites="tunstall2023zephyr"><a href="#ref-tunstall2023zephyr"
  role="doc-biblioref">[18]</a></span>, Tülu 2 <span class="citation" data-cites="ivison2023camels"><a href="#ref-ivison2023camels"
  role="doc-biblioref">[19]</a></span>, and many other models showed that the DPO era of post-training had begun. Chris Manning literally thanked me for “saving DPO.” This is how fine the margins are on evolutions of best practices with leading labs being locked down. Open post-training was cruising again.</p>
        <p>Preference-tuning was something you needed to do to meet the table stakes of releasing a good model since late 2023. The DPO era continued through 2024, in the form of never-ending variants on the algorithm, but we were very far into another slump in open recipes. Open post-training recipes had saturated the extent of knowledge and resources available.<br /> A year after Zephyr and Tulu 2, the same breakout dataset, UltraFeedback is arguably still state-of-the-art for preference tuning in open recipes <span class="citation" data-cites="cui2023ultrafeedback"><a href="#ref-cui2023ultrafeedback"
  role="doc-biblioref">[20]</a></span>.</p>
        <p>At the same time, the Llama 3.1 <span class="citation" data-cites="dubey2024llama"><a href="#ref-dubey2024llama"
  role="doc-biblioref">[21]</a></span> and Nemotron 4 340B <span class="citation" data-cites="adler2024nemotron"><a
  href="#ref-adler2024nemotron" role="doc-biblioref">[22]</a></span> reports gave us substantive hints that large-scale post-training is much more complex and impactful. The closed labs are doing full post-training – a large multi-stage process of instruction tuning, RLHF, prompt design, etc. – where academic papers are just scratching the surface. Tülu 3 represented a comprehensive, open effort to build the foundation of future academic post-training research <span class="citation" data-cites="lambert2024t"><a href="#ref-lambert2024t"
  role="doc-biblioref">[6]</a></span>.</p>
        <p>Today, post-training is a complex process involving the aforementioned training objectives applied in various orders in order to target specific capabilities. This book is designed to give a platform to understand all of these techniques, and in coming years the best practices for how to interleave them will emerge.</p>
        <p>The primary areas of innovation in post-training are now in reinforcement finetuning, reasoning training, and related ideas. This newer methods build extensively on the infrastructure and ideas of RLHF, but are evolving far faster. This book is written to capture the first stable literature for RLHF after its initial period of rapid change.
        </p>
        <h2 id="scope-of-this-book">Scope of This Book</h2>
        <p>This book hopes to touch on each of the core steps of doing canonical RLHF implementations. It will not cover all the history of the components nor recent research methods, just techniques, problems, and trade-offs that have been proven to occur again and again.</p>
        <h3 id="chapter-summaries">Chapter Summaries</h3>
        <p>This book has the following chapters:</p>
        <h4 id="introductions">Introductions</h4>
        <p>Reference material useful throughout the book.</p>
        <ol type="1">
            <li>Introduction: Overview of RLHF and what this book provides.</li>
            <li>Seminal (Recent) Works: Key models and papers in the history of RLHF techniques.</li>
            <li>Definitions: Mathematical definitions for RL, language modeling, and other ML techniques leveraged in this book.</li>
        </ol>
        <h4 id="problem-setup-context">Problem Setup &amp; Context</h4>
        <p>Context for the big picture problem RLHF is trying to solve.</p>
        <ol start="4" type="1">
            <li>RLHF Training Overview: How the training objective for RLHF is designed and basics of understanding it.</li>
            <li>What are preferences?: Why human preference data is needed to fuel and understand RLHF.</li>
            <li>Preference Data: How preference data is collected for RLHF.</li>
        </ol>
        <h4 id="optimization-tools">Optimization Tools</h4>
        <p>The suite of techniques used to optimize language models to align them to human preferences. This is a serial presentation of the techniques one can use to solve the problems proposed in the previous chapters.
        </p>
        <ol start="7" type="1">
            <li>Reward Modeling: Training reward models from preference data that act as an optimization target for RL training (or for use in data filtering).
            </li>
            <li>Regularization: Tools to constrain these optimization tools to effective regions of the parameter space.</li>
            <li>Instruction Tuning: Adapting language models to the question-answer format.</li>
            <li>Rejection Sampling: A basic technique for using a reward model with instruction tuning to align models.</li>
            <li>Policy Gradients: The core RL techniques used to optimize reward models (and other signals) throughout RLHF.</li>
            <li>Direct Alignment Algorithms: Algorithms that optimize the RLHF objective direction from pairwise preference data rather than learning a reward model first.</li>
        </ol>
        <h4 id="advanced-tbd">Advanced (TBD)</h4>
        <p>Newer RLHF techniques and discussions that are not clearly established, but are important to current generations of models.</p>
        <ol start="13" type="1">
            <li>Constitutional AI and AI Feedback</li>
            <li>Reasoning and Reinforcement Finetuning</li>
            <li>Synthetic Data</li>
            <li>Evaluation</li>
        </ol>
        <h4 id="open-questions-tbd">Open Questions (TBD)</h4>
        <p>Fundamental problems and discussions for the long-term evolution of how RLHF is used.</p>
        <ol start="16" type="1">
            <li>Over-optimization</li>
            <li>Style and Information</li>
        </ol>
        <h3 id="target-audience">Target Audience</h3>
        <p>This book is intended for audiences with entry level experience with language modeling, reinforcement learning, and general machine learning. It will not have exhaustive documentation for all the techniques, but just those crucial to understanding RLHF.</p>
        <h3 id="how-to-use-this-book">How to Use This Book</h3>
        <p>This book was largely created because there were no canonical references for important topics in the RLHF workflow. The contributions of this book are supposed to give you the minimum knowledge needed to try a toy implementation or dive into the literature. This is <em>not</em> a comprehensive textbook, but rather a quick book for reminders and getting started. Additionally, given the web-first nature of this book, it is expected that there are minor typos and somewhat random progressions – please contribute by fixing bugs or suggesting important content on <a href="https://github.com/natolambert/rlhf-book">GitHub</a>.</p>
        <h3 id="about-the-author">About the Author</h3>
        <p>Dr. Nathan Lambert is a RLHF researcher contributing to the open science of language model fine-tuning. He has released many models trained with RLHF, their subsequent datasets, and training codebases in his time at the Allen Institute for AI (Ai2) and HuggingFace. Examples include <a href="https://huggingface.co/HuggingFaceH4/zephyr-7b-beta">Zephyr-Beta</a>,
            <a href="https://huggingface.co/allenai/tulu-2-dpo-70b">Tulu 2</a>, <a href="https://huggingface.co/allenai/OLMo-7B-Instruct">OLMo</a>, <a href="https://github.com/huggingface/trl">TRL</a>, <a href="https://github.com/allenai/open-instruct">Open Instruct</a>, and many more. He has written extensively on RLHF, including <a href="https://www.interconnects.ai/t/rlhf">many blog posts</a> and <a href="https://scholar.google.com/citations?hl=en&amp;user=O4jW7BsAAAAJ&amp;view_op=list_works&amp;sortby=pubdate">academic
  papers</a>.</p>
        <h2 id="future-of-rlhf">Future of RLHF</h2>
        <p>With the investment in language modeling, many variations on the traditional RLHF methods emerged. RLHF colloquially has become synonymous with multiple overlapping approaches. RLHF is a subset of preference fine-tuning (PreFT) techniques, including Direct Alignment Algorithms (See Chapter 12). RLHF is the tool most associated with rapid progress in “post-training” of language models, which encompasses all training after the large-scale autoregressive training on primarily web data. This textbook is a broad overview of RLHF and its directly neighboring methods, such as instruction tuning and other implementation details needed to set up a model for RLHF training.</p>
        <p>As more successes of fine-tuning language models with RL emerge, such as OpenAI’s o1 reasoning models, RLHF will be seen as the bridge that enabled further investment of RL methods for fine-tuning large base models.</p>
        <!-- This is the first paragraph of the introduction chapter.

  ## First: Images

  This is the first subsection. Please, admire the gloriousnes of this seagull:

  ![A cool seagull.](images/seagull.png)

  A bigger seagull:

  ![A cool big seagull.](images/seagull.png){ width=320px }

  ## Second: Tables

  This is the second subsection.


  Please, check [First: Images] subsection.

  Please, check [this](#first-images) subsection.

  | Index | Name |
  | ----- | ---- |
  | 0     | AAA  |
  | 1     | BBB  |
  | ...   | ...  |

  Table: This is an example table.

  ## Third: Equations

  Formula example: $\mu = \sum_{i=0}^{N} \frac{x_i}{N}$

  Now, full size:

  $$\mu = \sum_{i=0}^{N} \frac{x_i}{N}$$

  And a code sample:

  ```rb
  def hello_world
    puts "hello world!"
  end

  hello_world
  ```

  Check these unicode characters: ǽß¢ð€đŋμ

  ## Fourth: Cross references

  These cross references are disabled by default. To enable them, check the
  _[Cross references](https://github.com/wikiti/pandoc-book-template#cross-references)_
  section on the README.md file.

  Here's a list of cross references:

  - Check @fig:seagull.
  - Check @tbl:table.
  - Check @eq:equation.

  ![A cool seagull](images/seagull.png){#fig:seagull}

  $$ y = mx + b $$ {#eq:equation}

  | Index | Name |
  | ----- | ---- |
  | 0     | AAA  |
  | 1     | BBB  |
  | ...   | ...  |

  Table: This is an example table. {#tbl:table} -->
        <h1 class="unnumbered" id="bibliography">Bibliography</h1>
        <div id="refs" class="references csl-bib-body" data-entry-spacing="0" role="list">
            <div id="ref-christiano2017deep" class="csl-entry" role="listitem">
                <div class="csl-left-margin">[1] </div>
                <div class="csl-right-inline">P. F. Christiano, J. Leike, T. Brown, M. Martic, S. Legg, and D. Amodei, <span>“Deep reinforcement learning
  from human preferences,”</span> <em>Advances in neural information
  processing systems</em>, vol. 30, 2017.</div>
            </div>
            <div id="ref-stiennon2020learning" class="csl-entry" role="listitem">
                <div class="csl-left-margin">[2] </div>
                <div class="csl-right-inline">N. Stiennon <em>et al.</em>, <span>“Learning
  to summarize with human feedback,”</span> <em>Advances in Neural
  Information Processing Systems</em>, vol. 33, pp. 3008–3021, 2020.
                </div>
            </div>
            <div id="ref-ouyang2022training" class="csl-entry" role="listitem">
                <div class="csl-left-margin">[3] </div>
                <div class="csl-right-inline">L. Ouyang <em>et al.</em>, <span>“Training
  language models to follow instructions with human feedback,”</span>
                    <em>Advances in neural information processing systems</em>, vol. 35, pp. 27730–27744, 2022.</div>
            </div>
            <div id="ref-nakano2021webgpt" class="csl-entry" role="listitem">
                <div class="csl-left-margin">[4] </div>
                <div class="csl-right-inline">R. Nakano <em>et al.</em>, <span>“Webgpt:
  Browser-assisted question-answering with human feedback,”</span>
                    <em>arXiv preprint arXiv:2112.09332</em>, 2021.</div>
            </div>
            <div id="ref-bai2022training" class="csl-entry" role="listitem">
                <div class="csl-left-margin">[5] </div>
                <div class="csl-right-inline">Y. Bai <em>et al.</em>, <span>“Training a
  helpful and harmless assistant with reinforcement learning from human
  feedback,”</span> <em>arXiv preprint arXiv:2204.05862</em>, 2022.
                </div>
            </div>
            <div id="ref-lambert2024t" class="csl-entry" role="listitem">
                <div class="csl-left-margin">[6] </div>
                <div class="csl-right-inline">N. Lambert <em>et al.</em>, <span>“T<span
  class="math inline">\(\backslash\)</span>" ULU 3: Pushing frontiers in open language model post-training,”</span> <em>arXiv preprint
  arXiv:2411.15124</em>, 2024.</div>
            </div>
            <div id="ref-kirk2023understanding" class="csl-entry" role="listitem">
                <div class="csl-left-margin">[7] </div>
                <div class="csl-right-inline">R. Kirk <em>et al.</em>, <span>“Understanding
  the effects of rlhf on llm generalisation and diversity,”</span>
                    <em>arXiv preprint arXiv:2310.06452</em>, 2023.</div>
            </div>
            <div id="ref-chu2025sft" class="csl-entry" role="listitem">
                <div class="csl-left-margin">[8] </div>
                <div class="csl-right-inline">T. Chu <em>et al.</em>, <span>“Sft memorizes,
  rl generalizes: A comparative study of foundation model
  post-training,”</span> <em>arXiv preprint arXiv:2501.17161</em>, 2025.
                </div>
            </div>
            <div id="ref-singhal2023long" class="csl-entry" role="listitem">
                <div class="csl-left-margin">[9] </div>
                <div class="csl-right-inline">P. Singhal, T. Goyal, J. Xu, and G. Durrett,
                    <span>“A long way to go: Investigating length correlations in
  rlhf,”</span> <em>arXiv preprint arXiv:2310.03716</em>, 2023.</div>
            </div>
            <div id="ref-park2024disentangling" class="csl-entry" role="listitem">
                <div class="csl-left-margin">[10] </div>
                <div class="csl-right-inline">R. Park, R. Rafailov, S. Ermon, and C. Finn,
                    <span>“Disentangling length from quality in direct preference
  optimization,”</span> <em>arXiv preprint arXiv:2403.19159</em>, 2024.
                </div>
            </div>
            <div id="ref-alpaca" class="csl-entry" role="listitem">
                <div class="csl-left-margin">[11] </div>
                <div class="csl-right-inline">R. Taori <em>et al.</em>, <span>“Stanford
  alpaca: An instruction-following LLaMA model,”</span> <em>GitHub
  repository</em>. <a href="https://github.com/tatsu-lab/stanford_alpaca" class="uri">https://github.com/tatsu-lab/stanford_alpaca</a>; GitHub, 2023.
                </div>
            </div>
            <div id="ref-vicuna2023" class="csl-entry" role="listitem">
                <div class="csl-left-margin">[12] </div>
                <div class="csl-right-inline">W.-L. Chiang <em>et al.</em>, <span>“Vicuna:
  An open-source chatbot impressing GPT-4 with 90%* ChatGPT
  quality.”</span> 2023. Available: <a href="https://lmsys.org/blog/2023-03-30-vicuna/">https://lmsys.org/blog/2023-03-30-vicuna/</a></div>
            </div>
            <div id="ref-koala_blogpost_2023" class="csl-entry" role="listitem">
                <div class="csl-left-margin">[13] </div>
                <div class="csl-right-inline">X. Geng <em>et al.</em>, <span>“Koala: A
  dialogue model for academic research.”</span> Blog post, 2023. Accessed: Apr. 03, 2023. [Online]. Available: <a href="https://bair.berkeley.edu/blog/2023/04/03/koala/">https://bair.berkeley.edu/blog/2023/04/03/koala/</a></div>
            </div>
            <div id="ref-DatabricksBlog2023DollyV1" class="csl-entry" role="listitem">
                <div class="csl-left-margin">[14] </div>
                <div class="csl-right-inline">M. Conover <em>et al.</em>, <span>“Hello
  dolly: Democratizing the magic of ChatGPT with open models.”</span> Accessed: Jun. 30, 2023. [Online]. Available: <a href="https://www.databricks.com/blog/2023/03/24/hello-dolly-democratizing-magic-chatgpt-open-models.html">https://www.databricks.com/blog/2023/03/24/hello-dolly-democratizing-magic-chatgpt-open-models.html</a></div>
            </div>
            <div id="ref-askell2021general" class="csl-entry" role="listitem">
                <div class="csl-left-margin">[15] </div>
                <div class="csl-right-inline">A. Askell <em>et al.</em>, <span>“A general
  language assistant as a laboratory for alignment,”</span> <em>arXiv
  preprint arXiv:2112.00861</em>, 2021.</div>
            </div>
            <div id="ref-bai2022constitutional" class="csl-entry" role="listitem">
                <div class="csl-left-margin">[16] </div>
                <div class="csl-right-inline">Y. Bai <em>et al.</em>, <span>“Constitutional
  ai: Harmlessness from ai feedback,”</span> <em>arXiv preprint
  arXiv:2212.08073</em>, 2022.</div>
            </div>
            <div id="ref-rafailov2024direct" class="csl-entry" role="listitem">
                <div class="csl-left-margin">[17] </div>
                <div class="csl-right-inline">R. Rafailov, A. Sharma, E. Mitchell, C. D. Manning, S. Ermon, and C. Finn, <span>“Direct preference optimization:
  Your language model is secretly a reward model,”</span> <em>Advances
  in Neural Information Processing Systems</em>, vol. 36, 2024.</div>
            </div>
            <div id="ref-tunstall2023zephyr" class="csl-entry" role="listitem">
                <div class="csl-left-margin">[18] </div>
                <div class="csl-right-inline">L. Tunstall <em>et al.</em>, <span>“Zephyr:
  Direct distillation of lm alignment,”</span> <em>arXiv preprint
  arXiv:2310.16944</em>, 2023.</div>
            </div>
            <div id="ref-ivison2023camels" class="csl-entry" role="listitem">
                <div class="csl-left-margin">[19] </div>
                <div class="csl-right-inline">H. Ivison <em>et al.</em>, <span>“Camels in a
  changing climate: Enhancing lm adaptation with tulu 2,”</span>
                    <em>arXiv preprint arXiv:2311.10702</em>, 2023.</div>
            </div>
            <div id="ref-cui2023ultrafeedback" class="csl-entry" role="listitem">
                <div class="csl-left-margin">[20] </div>
                <div class="csl-right-inline">G. Cui <em>et al.</em>, <span>“Ultrafeedback:
  Boosting language models with high-quality feedback,”</span> 2023.
                </div>
            </div>
            <div id="ref-dubey2024llama" class="csl-entry" role="listitem">
                <div class="csl-left-margin">[21] </div>
                <div class="csl-right-inline">A. Dubey <em>et al.</em>, <span>“The llama 3
  herd of models,”</span> <em>arXiv preprint arXiv:2407.21783</em>, 2024.
                </div>
            </div>
            <div id="ref-adler2024nemotron" class="csl-entry" role="listitem">
                <div class="csl-left-margin">[22] </div>
                <div class="csl-right-inline">B. Adler <em>et al.</em>, <span>“Nemotron-4
  340B technical report,”</span> <em>arXiv preprint
  arXiv:2406.11704</em>, 2024.</div>
            </div>
        </div>
    </div>

    <div id="chapter-navigation" style="display: flex; justify-content: space-between; padding: 2em 0;">
        <a href="https://rlhfbook.com/" class="prev-chapter">
    ← Previous: Home
  </a>

        <a href="02-related-works.html" class="next-chapter">
    Next: Key Related Works →
  </a>
    </div>

    <footer style="padding: 20px; text-align: center;">
        <hr> Citation <br>
        <div style="text-align: left; font-size: small; color: #888;">
            @book{rlhf2024,<br> &nbsp;&nbsp;author = {Nathan Lambert},<br> &nbsp;&nbsp;title = {Reinforcement Learning from Human Feedback},<br> &nbsp;&nbsp;year = {2024},<br> &nbsp;&nbsp;publisher = {Online},<br> &nbsp;&nbsp;url = {https://rlhfbook.com},<br> }
        </div>
        <div>
            <a href="https://github.com/natolambert/rlhf-book" target="_blank">
                <img src="https://github.githubassets.com/images/modules/logos_page/GitHub-Mark.png" alt="GitHub" style="width: 40px; height: 40px;">
            </a>
            <!-- Add more social links here -->
        </div>
        <p>&copy; 2024 RLHF Book Team</p>
    </footer>
</body>

</html>
"###;

#[test]
fn test_extraction_from_hn_homepage() {
    use crate::engine::engine::get_test_engine;
    use crate::engine::node::{NodeLabel, Payload};
    use crate::engine::EdgeLabel;
    use crate::entity::web::link::Link;
    use std::sync::Arc;

    let test_engine = get_test_engine();
    let arced_test_engine = Arc::new(&test_engine);
    let link_node_id = Link::add(
        arced_test_engine,
        &"https://news.ycombinator.com".to_string(),
        vec![NodeLabel::AddedByUser, NodeLabel::Link],
        vec![],
        true,
    )
    .unwrap();

    let webpage_node_id = test_engine
        .get_or_add_node(
            Payload::Text(HN_HOMEPAGE.to_string()),
            vec![NodeLabel::Content, NodeLabel::WebPage],
            true,
            None,
        )
        .unwrap()
        .get_node_id();
    test_engine
        .add_connection(
            (link_node_id, webpage_node_id.clone()),
            (EdgeLabel::PathOf, EdgeLabel::ContentOf),
        )
        .unwrap();
    test_engine.process_nodes();

    let children_of_webpage = test_engine
        .get_node_ids_connected_with_label(&webpage_node_id, &EdgeLabel::ParentOf)
        .unwrap();
    assert_eq!(children_of_webpage.len(), 222);

    let first_child_of_webpage = test_engine
        .get_node_by_id(children_of_webpage.get(0).unwrap())
        .unwrap();
    assert_eq!(
        first_child_of_webpage.labels,
        vec![NodeLabel::Title, NodeLabel::Partial]
    );

    // Count the number of Link nodes
    let link_node_ids = test_engine.get_node_ids_with_label(&NodeLabel::Link);
    assert_eq!(link_node_ids.len(), 222);
}

#[test]
fn test_extract_data_only_from_specified_links() {
    use crate::engine::engine::get_test_engine;
    use crate::engine::node::{NodeLabel, Payload};
    use crate::engine::EdgeLabel;
    use crate::entity::project_settings::ProjectSettings;
    use crate::entity::web::link::Link;
    use std::sync::Arc;

    let test_engine = get_test_engine();
    let arced_test_engine = Arc::new(&test_engine);

    let project_settings_node_id = arced_test_engine
        .get_or_add_node(
            Payload::ProjectSettings(ProjectSettings {
                only_extract_data_from_specified_links: true,
                ..Default::default()
            }),
            vec![NodeLabel::AddedByUser, NodeLabel::ProjectSettings],
            true,
            None,
        )
        .unwrap()
        .get_node_id();

    let link_node_id = Link::add(
        arced_test_engine.clone(),
        &"https://news.ycombinator.com".to_string(),
        vec![NodeLabel::AddedByUser, NodeLabel::Link],
        vec![],
        true,
    )
    .unwrap();

    arced_test_engine
        .add_connection(
            (project_settings_node_id, link_node_id),
            (EdgeLabel::RelatedTo, EdgeLabel::RelatedTo),
        )
        .unwrap();

    let webpage_node_id = test_engine
        .get_or_add_node(
            Payload::Text(HN_HOMEPAGE.to_string()),
            vec![NodeLabel::Content, NodeLabel::WebPage],
            true,
            None,
        )
        .unwrap()
        .get_node_id();
    test_engine
        .add_connection(
            (link_node_id, webpage_node_id.clone()),
            (EdgeLabel::PathOf, EdgeLabel::ContentOf),
        )
        .unwrap();
    test_engine.process_nodes();

    let children_of_webpage = test_engine
        .get_node_ids_connected_with_label(&webpage_node_id, &EdgeLabel::ParentOf)
        .unwrap();
    assert_eq!(children_of_webpage.len(), 1);

    let first_child_of_webpage = test_engine
        .get_node_by_id(children_of_webpage.get(0).unwrap())
        .unwrap();
    assert_eq!(
        first_child_of_webpage.labels,
        vec![NodeLabel::Title, NodeLabel::Partial]
    );

    // Count the number of Link nodes
    let link_node_ids = test_engine.get_node_ids_with_label(&NodeLabel::Link);
    assert_eq!(link_node_ids.len(), 1);
}

#[test]
fn test_crawl_within_domains_of_specified_links() {
    use crate::engine::engine::get_test_engine;
    use crate::engine::node::{NodeLabel, Payload};
    use crate::engine::EdgeLabel;
    use crate::entity::project_settings::ProjectSettings;
    use crate::entity::web::link::Link;
    use std::sync::Arc;

    let test_engine = get_test_engine();
    let arced_test_engine = Arc::new(&test_engine);

    let project_settings_node_id = arced_test_engine
        .get_or_add_node(
            Payload::ProjectSettings(ProjectSettings {
                only_crawl_within_domains_of_specified_links: true,
                ..Default::default()
            }),
            vec![NodeLabel::AddedByUser, NodeLabel::ProjectSettings],
            true,
            None,
        )
        .unwrap()
        .get_node_id();

    let link_node_id = Link::add(
        arced_test_engine.clone(),
        &"https://news.ycombinator.com".to_string(),
        vec![NodeLabel::AddedByUser, NodeLabel::Link],
        vec![],
        true,
    )
    .unwrap();

    arced_test_engine
        .add_connection(
            (project_settings_node_id, link_node_id),
            (EdgeLabel::RelatedTo, EdgeLabel::RelatedTo),
        )
        .unwrap();

    let webpage_node_id = test_engine
        .get_or_add_node(
            Payload::Text(HN_HOMEPAGE.to_string()),
            vec![NodeLabel::Content, NodeLabel::WebPage],
            true,
            None,
        )
        .unwrap()
        .get_node_id();
    test_engine
        .add_connection(
            (link_node_id, webpage_node_id.clone()),
            (EdgeLabel::PathOf, EdgeLabel::ContentOf),
        )
        .unwrap();

    // We process only once, so scraped links are not fetched
    test_engine.process_nodes();

    let children_of_webpage = test_engine
        .get_node_ids_connected_with_label(&webpage_node_id, &EdgeLabel::ParentOf)
        .unwrap();
    assert_eq!(children_of_webpage.len(), 191);

    let first_child_of_webpage = test_engine
        .get_node_by_id(children_of_webpage.get(0).unwrap())
        .unwrap();
    assert_eq!(
        first_child_of_webpage.labels,
        vec![NodeLabel::Title, NodeLabel::Partial]
    );

    // Count the number of Link nodes
    let link_node_ids = test_engine.get_node_ids_with_label(&NodeLabel::Link);
    assert_eq!(link_node_ids.len(), 189);

    // Check that there is only one Domain node
    let domain_node_ids = test_engine.get_node_ids_with_label(&NodeLabel::Domain);
    assert_eq!(domain_node_ids.len(), 1);

    let edges_from_domain_node = test_engine
        .get_node_ids_connected_with_label(domain_node_ids.get(0).unwrap(), &EdgeLabel::OwnerOf)
        .unwrap();
    assert_eq!(edges_from_domain_node.len(), 189);
}

const HN_HOMEPAGE: &str = r###"
<html lang="en" op="news">

<head>
    <meta name="referrer" content="origin">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <link rel="stylesheet" type="text/css" href="news.css?0Cm7vITOnsFHYgoWtXzU">
    <link rel="icon" href="y18.svg">
    <link rel="alternate" type="application/rss+xml" title="RSS" href="rss">
    <title>Hacker News</title>
</head>

<body>
    <center>
        <table id="hnmain" border="0" cellpadding="0" cellspacing="0" width="85%" bgcolor="#f6f6ef">
            <tr>
                <td bgcolor="#ff6600">
                    <table border="0" cellpadding="0" cellspacing="0" width="100%" style="padding:2px">
                        <tr>
                            <td style="width:18px;padding-right:4px">
                                <a href="https://news.ycombinator.com"><img src="y18.svg" width="18" height="18" style="border:1px white solid; display:block"></a>
                            </td>
                            <td style="line-height:12pt; height:10px;"><span class="pagetop"><b class="hnname"><a href="news">Hacker News</a></b>
                            <a href="newest">new</a> | <a href="threads?id=brainless">threads</a> | <a href="front">past</a> | <a href="newcomments">comments</a> | <a href="ask">ask</a> | <a href="show">show</a> | <a href="jobs">jobs</a> | <a href="submit" rel="nofollow">submit</a>            </span></td>
                            <td style="text-align:right;padding-right:4px;"><span class="pagetop">
                              <a id='me' href="user?id=brainless">brainless</a> (<span id='karma'>2062</span>) | <a id='logout' rel='nofollow' href="logout?auth=d5d64de07c4221178f581318827a255df567ce8f&amp;goto=news">logout</a> </span>
                            </td>
                        </tr>
                    </table>
                </td>
            </tr>
            <tr id="pagespace" title="" style="height:10px"></tr>
            <tr>
                <td>
                    <table border="0" cellpadding="0" cellspacing="0">
                        <tr class='athing submission' id='43295692'>
                            <td align="right" valign="top" class="title"><span class="rank">1.</span></td>
                            <td valign="top" class="votelinks">
                                <center>
                                    <a id='up_43295692' class='clicky' href='vote?id=43295692&amp;how=up&amp;auth=02848849445061f734058ad2a846c41c9732f8fd&amp;goto=news'>
                                        <div class='votearrow' title='upvote'></div>
                                    </a>
                                </center>
                            </td>
                            <td class="title"><span class="titleline"><a href="https://www.nature.com/articles/d41586-025-00648-5">AI tools are spotting errors in research papers: inside a growing movement</a><span class="sitebit comhead"> (<a href="from?site=nature.com"><span class="sitestr">nature.com</span></a>)</span>
                                </span>
                            </td>
                        </tr>
                        <tr>
                            <td colspan="2"></td>
                            <td class="subtext"><span class="subline">
          <span class="score" id="score_43295692">56 points</span> by <a href="user?id=kgwgk" class="hnuser">kgwgk</a> <span class="age" title="2025-03-07T22:54:58 1741388098"><a href="item?id=43295692">1 hour ago</a></span> <span id="unv_43295692"></span> | <a href="flag?id=43295692&amp;auth=02848849445061f734058ad2a846c41c9732f8fd&amp;goto=news" rel="nofollow">flag</a> | <a href="hide?id=43295692&amp;auth=02848849445061f734058ad2a846c41c9732f8fd&amp;goto=news" class="clicky hider">hide</a> | <a href="item?id=43295692">24&nbsp;comments</a> </span>
                            </td>
                        </tr>
                        <tr class="spacer" style="height:5px"></tr>
                        <tr class='athing submission' id='43300528'>
                            <td align="right" valign="top" class="title"><span class="rank">2.</span></td>
                            <td valign="top" class="votelinks">
                                <center>
                                    <a id='up_43300528' class='clicky' href='vote?id=43300528&amp;how=up&amp;auth=688e49a9e9787b5ab0379b98411bcbe0f5f6b7a7&amp;goto=news'>
                                        <div class='votearrow' title='upvote'></div>
                                    </a>
                                </center>
                            </td>
                            <td class="title"><span class="titleline"><a href="https://www.scattered-thoughts.net/writing/the-program-is-the-database-is-the-interface/">The program is the database is the interface</a><span class="sitebit comhead"> (<a href="from?site=scattered-thoughts.net"><span class="sitestr">scattered-thoughts.net</span></a>)</span>
                                </span>
                            </td>
                        </tr>
                        <tr>
                            <td colspan="2"></td>
                            <td class="subtext"><span class="subline">
          <span class="score" id="score_43300528">9 points</span> by <a href="user?id=tosh" class="hnuser">tosh</a> <span class="age" title="2025-03-08T14:31:23 1741444283"><a href="item?id=43300528">1 hour ago</a></span> <span id="unv_43300528"></span> | <a href="flag?id=43300528&amp;auth=688e49a9e9787b5ab0379b98411bcbe0f5f6b7a7&amp;goto=news" rel="nofollow">flag</a> | <a href="hide?id=43300528&amp;auth=688e49a9e9787b5ab0379b98411bcbe0f5f6b7a7&amp;goto=news" class="clicky hider">hide</a> | <a href="item?id=43300528">discuss</a> </span>
                            </td>
                        </tr>
                        <tr class="spacer" style="height:5px"></tr>
                        <tr class='athing submission' id='43297574'>
                            <td align="right" valign="top" class="title"><span class="rank">3.</span></td>
                            <td valign="top" class="votelinks">
                                <center>
                                    <a id='up_43297574' class='clicky' href='vote?id=43297574&amp;how=up&amp;auth=8b48165aa71a9c2629127858299ad2f28c8c5087&amp;goto=news'>
                                        <div class='votearrow' title='upvote'></div>
                                    </a>
                                </center>
                            </td>
                            <td class="title"><span class="titleline"><a href="https://typesanitizer.com/blog/errors.html">An epic treatise on error models for systems programming languages</a><span class="sitebit comhead"> (<a href="from?site=typesanitizer.com"><span class="sitestr">typesanitizer.com</span></a>)</span>
                                </span>
                            </td>
                        </tr>
                        <tr>
                            <td colspan="2"></td>
                            <td class="subtext"><span class="subline">
          <span class="score" id="score_43297574">134 points</span> by <a href="user?id=transpute" class="hnuser">transpute</a> <span class="age" title="2025-03-08T04:46:33 1741409193"><a href="item?id=43297574">11 hours ago</a></span> <span id="unv_43297574"></span> | <a href="flag?id=43297574&amp;auth=8b48165aa71a9c2629127858299ad2f28c8c5087&amp;goto=news" rel="nofollow">flag</a> | <a href="hide?id=43297574&amp;auth=8b48165aa71a9c2629127858299ad2f28c8c5087&amp;goto=news" class="clicky hider">hide</a> | <a href="item?id=43297574">36&nbsp;comments</a> </span>
                            </td>
                        </tr>
                        <tr class="spacer" style="height:5px"></tr>
                        <tr class='athing submission' id='43296918'>
                            <td align="right" valign="top" class="title"><span class="rank">4.</span></td>
                            <td valign="top" class="votelinks">
                                <center>
                                    <a id='up_43296918' class='clicky' href='vote?id=43296918&amp;how=up&amp;auth=445b29e6224ce90eae9d0aa78f826ef7db00e8ae&amp;goto=news'>
                                        <div class='votearrow' title='upvote'></div>
                                    </a>
                                </center>
                            </td>
                            <td class="title"><span class="titleline"><a href="https://rlama.dev/">Show HN: Open-Source DocumentAI with Ollama</a><span class="sitebit comhead"> (<a href="from?site=rlama.dev"><span class="sitestr">rlama.dev</span></a>)</span>
                                </span>
                            </td>
                        </tr>
                        <tr>
                            <td colspan="2"></td>
                            <td class="subtext"><span class="subline">
          <span class="score" id="score_43296918">212 points</span> by <a href="user?id=Dontizi" class="hnuser">Dontizi</a> <span class="age" title="2025-03-08T02:12:13 1741399933"><a href="item?id=43296918">13 hours ago</a></span> <span id="unv_43296918"></span> | <a href="flag?id=43296918&amp;auth=445b29e6224ce90eae9d0aa78f826ef7db00e8ae&amp;goto=news" rel="nofollow">flag</a> | <a href="hide?id=43296918&amp;auth=445b29e6224ce90eae9d0aa78f826ef7db00e8ae&amp;goto=news" class="clicky hider">hide</a> | <a href="item?id=43296918">26&nbsp;comments</a> </span>
                            </td>
                        </tr>
                        <tr class="spacer" style="height:5px"></tr>
                        <tr class='athing submission' id='43258670'>
                            <td align="right" valign="top" class="title"><span class="rank">5.</span></td>
                            <td valign="top" class="votelinks">
                                <center>
                                    <a id='up_43258670' class='clicky' href='vote?id=43258670&amp;how=up&amp;auth=8136d5d0542e7b89e6cd0a20ff11dd4e9d96a451&amp;goto=news'>
                                        <div class='votearrow' title='upvote'></div>
                                    </a>
                                </center>
                            </td>
                            <td class="title"><span class="titleline"><a href="https://news.cornell.edu/stories/2025/03/ai-models-makes-precise-copies-cuneiform-characters">AI models makes precise copies of cuneiform characters</a><span class="sitebit comhead"> (<a href="from?site=cornell.edu"><span class="sitestr">cornell.edu</span></a>)</span>
                                </span>
                            </td>
                        </tr>
                        <tr>
                            <td colspan="2"></td>
                            <td class="subtext"><span class="subline">
          <span class="score" id="score_43258670">32 points</span> by <a href="user?id=geox" class="hnuser">geox</a> <span class="age" title="2025-03-04T19:01:20 1741114880"><a href="item?id=43258670">7 hours ago</a></span> <span id="unv_43258670"></span> | <a href="flag?id=43258670&amp;auth=8136d5d0542e7b89e6cd0a20ff11dd4e9d96a451&amp;goto=news" rel="nofollow">flag</a> | <a href="hide?id=43258670&amp;auth=8136d5d0542e7b89e6cd0a20ff11dd4e9d96a451&amp;goto=news" class="clicky hider">hide</a> | <a href="item?id=43258670">5&nbsp;comments</a> </span>
                            </td>
                        </tr>
                        <tr class="spacer" style="height:5px"></tr>
                        <tr class='athing submission' id='43297590'>
                            <td align="right" valign="top" class="title"><span class="rank">6.</span></td>
                            <td valign="top" class="votelinks">
                                <center>
                                    <a id='up_43297590' class='clicky' href='vote?id=43297590&amp;how=up&amp;auth=b352b134112dab5ed095a4f2ad1d6bf963365520&amp;goto=news'>
                                        <div class='votearrow' title='upvote'></div>
                                    </a>
                                </center>
                            </td>
                            <td class="title"><span class="titleline"><a href="https://www.falkon.org">Falkon: A KDE Web Browser</a><span class="sitebit comhead"> (<a href="from?site=falkon.org"><span class="sitestr">falkon.org</span></a>)</span>
                                </span>
                            </td>
                        </tr>
                        <tr>
                            <td colspan="2"></td>
                            <td class="subtext"><span class="subline">
          <span class="score" id="score_43297590">84 points</span> by <a href="user?id=0x54MUR41" class="hnuser">0x54MUR41</a> <span class="age" title="2025-03-08T04:51:25 1741409485"><a href="item?id=43297590">10 hours ago</a></span> <span id="unv_43297590"></span> | <a href="flag?id=43297590&amp;auth=b352b134112dab5ed095a4f2ad1d6bf963365520&amp;goto=news" rel="nofollow">flag</a> | <a href="hide?id=43297590&amp;auth=b352b134112dab5ed095a4f2ad1d6bf963365520&amp;goto=news" class="clicky hider">hide</a> | <a href="item?id=43297590">32&nbsp;comments</a> </span>
                            </td>
                        </tr>
                        <tr class="spacer" style="height:5px"></tr>
                        <tr class='athing submission' id='43299815'>
                            <td align="right" valign="top" class="title"><span class="rank">7.</span></td>
                            <td valign="top" class="votelinks">
                                <center>
                                    <a id='up_43299815' class='clicky' href='vote?id=43299815&amp;how=up&amp;auth=f6020b07909b71064732eb88f8d5ba3c8157d8e4&amp;goto=news'>
                                        <div class='votearrow' title='upvote'></div>
                                    </a>
                                </center>
                            </td>
                            <td class="title"><span class="titleline"><a href="https://contraptions.venkateshrao.com/p/discworld-rules">Discworld Rules</a><span class="sitebit comhead"> (<a href="from?site=venkateshrao.com"><span class="sitestr">venkateshrao.com</span></a>)</span>
                                </span>
                            </td>
                        </tr>
                        <tr>
                            <td colspan="2"></td>
                            <td class="subtext"><span class="subline">
          <span class="score" id="score_43299815">76 points</span> by <a href="user?id=jger15" class="hnuser">jger15</a> <span class="age" title="2025-03-08T12:48:11 1741438091"><a href="item?id=43299815">2 hours ago</a></span> <span id="unv_43299815"></span> | <a href="flag?id=43299815&amp;auth=f6020b07909b71064732eb88f8d5ba3c8157d8e4&amp;goto=news" rel="nofollow">flag</a> | <a href="hide?id=43299815&amp;auth=f6020b07909b71064732eb88f8d5ba3c8157d8e4&amp;goto=news" class="clicky hider">hide</a> | <a href="item?id=43299815">69&nbsp;comments</a> </span>
                            </td>
                        </tr>
                        <tr class="spacer" style="height:5px"></tr>
                        <tr class='athing submission' id='43257927'>
                            <td align="right" valign="top" class="title"><span class="rank">8.</span></td>
                            <td valign="top" class="votelinks">
                                <center>
                                    <a id='up_43257927' class='clicky' href='vote?id=43257927&amp;how=up&amp;auth=8e2875fd70726bc81c54b8f26a57b7299649f8be&amp;goto=news'>
                                        <div class='votearrow' title='upvote'></div>
                                    </a>
                                </center>
                            </td>
                            <td class="title"><span class="titleline"><a href="https://bodydoubling.com/">Body Doubling</a><span class="sitebit comhead"> (<a href="from?site=bodydoubling.com"><span class="sitestr">bodydoubling.com</span></a>)</span>
                                </span>
                            </td>
                        </tr>
                        <tr>
                            <td colspan="2"></td>
                            <td class="subtext"><span class="subline">
          <span class="score" id="score_43257927">9 points</span> by <a href="user?id=yamrzou" class="hnuser">yamrzou</a> <span class="age" title="2025-03-04T17:54:54 1741110894"><a href="item?id=43257927">4 hours ago</a></span> <span id="unv_43257927"></span> | <a href="flag?id=43257927&amp;auth=8e2875fd70726bc81c54b8f26a57b7299649f8be&amp;goto=news" rel="nofollow">flag</a> | <a href="hide?id=43257927&amp;auth=8e2875fd70726bc81c54b8f26a57b7299649f8be&amp;goto=news" class="clicky hider">hide</a> | <a href="item?id=43257927">3&nbsp;comments</a> </span>
                            </td>
                        </tr>
                        <tr class="spacer" style="height:5px"></tr>
                        <tr class='athing submission' id='43257506'>
                            <td align="right" valign="top" class="title"><span class="rank">9.</span></td>
                            <td valign="top" class="votelinks">
                                <center>
                                    <a id='up_43257506' class='clicky' href='vote?id=43257506&amp;how=up&amp;auth=6be3120885fffce2946008d287e5673e9a11fd5f&amp;goto=news'>
                                        <div class='votearrow' title='upvote'></div>
                                    </a>
                                </center>
                            </td>
                            <td class="title"><span class="titleline"><a href="https://jdhwilkins.com/mountains-cliffs-and-caves-a-comprehensive-guide-to-using-perlin-noise-for-procedural-generation/">Mountains, Cliffs, and Caves: A Guide to Using Perlin Noise for Procedural Gen</a><span class="sitebit comhead"> (<a href="from?site=jdhwilkins.com"><span class="sitestr">jdhwilkins.com</span></a>)</span>
                                </span>
                            </td>
                        </tr>
                        <tr>
                            <td colspan="2"></td>
                            <td class="subtext"><span class="subline">
          <span class="score" id="score_43257506">129 points</span> by <a href="user?id=chwolfe" class="hnuser">chwolfe</a> <span class="age" title="2025-03-04T17:19:26 1741108766"><a href="item?id=43257506">15 hours ago</a></span> <span id="unv_43257506"></span> | <a href="flag?id=43257506&amp;auth=6be3120885fffce2946008d287e5673e9a11fd5f&amp;goto=news" rel="nofollow">flag</a> | <a href="hide?id=43257506&amp;auth=6be3120885fffce2946008d287e5673e9a11fd5f&amp;goto=news" class="clicky hider">hide</a> | <a href="item?id=43257506">25&nbsp;comments</a> </span>
                            </td>
                        </tr>
                        <tr class="spacer" style="height:5px"></tr>
                        <tr class='athing submission' id='43299659'>
                            <td align="right" valign="top" class="title"><span class="rank">10.</span></td>
                            <td valign="top" class="votelinks">
                                <center>
                                    <a id='up_43299659' class='clicky' href='vote?id=43299659&amp;how=up&amp;auth=f55ebd13883c68f9165466afa4b21a2a71734baf&amp;goto=news'>
                                        <div class='votearrow' title='upvote'></div>
                                    </a>
                                </center>
                            </td>
                            <td class="title"><span class="titleline"><a href="https://tzx.notion.site/What-I-Learned-Building-a-Free-Semantic-Search-Tool-for-GitHub-and-Why-I-Failed-1a09b742c7918033b318f3a5d7dc9751" rel="nofollow">Long Read: Lessons from Building Semantic Search for GitHub and Why I Failed</a><span class="sitebit comhead"> (<a href="from?site=tzx.notion.site"><span class="sitestr">tzx.notion.site</span></a>)</span>
                                </span>
                            </td>
                        </tr>
                        <tr>
                            <td colspan="2"></td>
                            <td class="subtext"><span class="subline">
          <span class="score" id="score_43299659">9 points</span> by <a href="user?id=zxt_tzx" class="hnuser">zxt_tzx</a> <span class="age" title="2025-03-08T12:23:46 1741436626"><a href="item?id=43299659">3 hours ago</a></span> <span id="unv_43299659"></span> | <a href="flag?id=43299659&amp;auth=f55ebd13883c68f9165466afa4b21a2a71734baf&amp;goto=news" rel="nofollow">flag</a> | <a href="hide?id=43299659&amp;auth=f55ebd13883c68f9165466afa4b21a2a71734baf&amp;goto=news" class="clicky hider">hide</a> | <a href="item?id=43299659">4&nbsp;comments</a> </span>
                            </td>
                        </tr>
                        <tr class="spacer" style="height:5px"></tr>
                        <tr class='athing submission' id='43299508'>
                            <td align="right" valign="top" class="title"><span class="rank">11.</span></td>
                            <td><img src="s.gif" height="1" width="14"></td>
                            <td class="title"><span class="titleline"><a href="https://jobs.ashbyhq.com/extend/9d4d8974-bd9b-432d-84ec-8268e5a8ed37" rel="nofollow">Extend (YC W23) is hiring engineers to build LLM document processing</a><span class="sitebit comhead"> (<a href="from?site=ashbyhq.com"><span class="sitestr">ashbyhq.com</span></a>)</span>
                                </span>
                            </td>
                        </tr>
                        <tr>
                            <td colspan="2"></td>
                            <td class="subtext">
                                <span class="age" title="2025-03-08T12:00:45 1741435245"><a href="item?id=43299508">3 hours ago</a></span> | <a href="hide?id=43299508&amp;auth=d16577b95b7e9e971f6d5cb1094b002bf1d4356f&amp;goto=news" class="clicky hider">hide</a> </td>
                        </tr>
                        <tr class="spacer" style="height:5px"></tr>
                        <tr class='athing submission' id='43277550'>
                            <td align="right" valign="top" class="title"><span class="rank">12.</span></td>
                            <td valign="top" class="votelinks">
                                <center>
                                    <a id='up_43277550' class='clicky' href='vote?id=43277550&amp;how=up&amp;auth=8ff5b6050d6b0dae718ee31fa4dadc33033c52c6&amp;goto=news'>
                                        <div class='votearrow' title='upvote'></div>
                                    </a>
                                </center>
                            </td>
                            <td class="title"><span class="titleline"><a href="https://www.newscientist.com/article/2467491-forces-deep-underground-seem-to-be-deforming-earths-inner-core/">Forces deep underground seem to be deforming Earth&#x27;s inner core</a><span class="sitebit comhead"> (<a href="from?site=newscientist.com"><span class="sitestr">newscientist.com</span></a>)</span>
                                </span>
                            </td>
                        </tr>
                        <tr>
                            <td colspan="2"></td>
                            <td class="subtext"><span class="subline">
          <span class="score" id="score_43277550">20 points</span> by <a href="user?id=walterbell" class="hnuser">walterbell</a> <span class="age" title="2025-03-06T07:46:50 1741247210"><a href="item?id=43277550">7 hours ago</a></span> <span id="unv_43277550"></span> | <a href="flag?id=43277550&amp;auth=8ff5b6050d6b0dae718ee31fa4dadc33033c52c6&amp;goto=news" rel="nofollow">flag</a> | <a href="hide?id=43277550&amp;auth=8ff5b6050d6b0dae718ee31fa4dadc33033c52c6&amp;goto=news" class="clicky hider">hide</a> | <a href="item?id=43277550">7&nbsp;comments</a> </span>
                            </td>
                        </tr>
                        <tr class="spacer" style="height:5px"></tr>
                        <tr class='athing submission' id='43292056'>
                            <td align="right" valign="top" class="title"><span class="rank">13.</span></td>
                            <td valign="top" class="votelinks">
                                <center>
                                    <a id='up_43292056' class='clicky' href='vote?id=43292056&amp;how=up&amp;auth=0fdce185f14b2318b528d46859d9c0e462b2542f&amp;goto=news'>
                                        <div class='votearrow' title='upvote'></div>
                                    </a>
                                </center>
                            </td>
                            <td class="title"><span class="titleline"><a href="https://developer.chrome.com/blog/command-and-commandfor">Introducing command And commandfor In HTML</a><span class="sitebit comhead"> (<a href="from?site=chrome.com"><span class="sitestr">chrome.com</span></a>)</span>
                                </span>
                            </td>
                        </tr>
                        <tr>
                            <td colspan="2"></td>
                            <td class="subtext"><span class="subline">
          <span class="score" id="score_43292056">373 points</span> by <a href="user?id=Kerrick" class="hnuser">Kerrick</a> <span class="age" title="2025-03-07T17:24:02 1741368242"><a href="item?id=43292056">22 hours ago</a></span> <span id="unv_43292056"></span> | <a href="flag?id=43292056&amp;auth=0fdce185f14b2318b528d46859d9c0e462b2542f&amp;goto=news" rel="nofollow">flag</a> | <a href="hide?id=43292056&amp;auth=0fdce185f14b2318b528d46859d9c0e462b2542f&amp;goto=news" class="clicky hider">hide</a> | <a href="item?id=43292056">218&nbsp;comments</a> </span>
                            </td>
                        </tr>
                        <tr class="spacer" style="height:5px"></tr>
                        <tr class='athing submission' id='43261593'>
                            <td align="right" valign="top" class="title"><span class="rank">14.</span></td>
                            <td valign="top" class="votelinks">
                                <center>
                                    <a id='up_43261593' class='clicky' href='vote?id=43261593&amp;how=up&amp;auth=aca40da984707cea6861f3aad8531680de9e2a24&amp;goto=news'>
                                        <div class='votearrow' title='upvote'></div>
                                    </a>
                                </center>
                            </td>
                            <td class="title"><span class="titleline"><a href="https://explore.sutrotower.com" rel="nofollow">Explore Sutro Tower</a><span class="sitebit comhead"> (<a href="from?site=sutrotower.com"><span class="sitestr">sutrotower.com</span></a>)</span>
                                </span>
                            </td>
                        </tr>
                        <tr>
                            <td colspan="2"></td>
                            <td class="subtext"><span class="subline">
          <span class="score" id="score_43261593">7 points</span> by <a href="user?id=archagon" class="hnuser">archagon</a> <span class="age" title="2025-03-05T01:30:07 1741138207"><a href="item?id=43261593">3 hours ago</a></span> <span id="unv_43261593"></span> | <a href="flag?id=43261593&amp;auth=aca40da984707cea6861f3aad8531680de9e2a24&amp;goto=news" rel="nofollow">flag</a> | <a href="hide?id=43261593&amp;auth=aca40da984707cea6861f3aad8531680de9e2a24&amp;goto=news" class="clicky hider">hide</a> | <a href="item?id=43261593">4&nbsp;comments</a> </span>
                            </td>
                        </tr>
                        <tr class="spacer" style="height:5px"></tr>
                        <tr class='athing submission' id='43299772'>
                            <td align="right" valign="top" class="title"><span class="rank">15.</span></td>
                            <td valign="top" class="votelinks">
                                <center>
                                    <a id='up_43299772' class='clicky' href='vote?id=43299772&amp;how=up&amp;auth=0ef4fe5be5852ac404e91e1caca1ad89cb77b5da&amp;goto=news'>
                                        <div class='votearrow' title='upvote'></div>
                                    </a>
                                </center>
                            </td>
                            <td class="title"><span class="titleline"><a href="https://www.theguardian.com/science/2025/mar/07/gene-edited-non-browning-banana-cut-food-waste-tropic-norwich">Gene-edited non-browning banana could cut food waste</a><span class="sitebit comhead"> (<a href="from?site=theguardian.com"><span class="sitestr">theguardian.com</span></a>)</span>
                                </span>
                            </td>
                        </tr>
                        <tr>
                            <td colspan="2"></td>
                            <td class="subtext"><span class="subline">
          <span class="score" id="score_43299772">21 points</span> by <a href="user?id=geox" class="hnuser">geox</a> <span class="age" title="2025-03-08T12:41:56 1741437716"><a href="item?id=43299772">3 hours ago</a></span> <span id="unv_43299772"></span> | <a href="flag?id=43299772&amp;auth=0ef4fe5be5852ac404e91e1caca1ad89cb77b5da&amp;goto=news" rel="nofollow">flag</a> | <a href="hide?id=43299772&amp;auth=0ef4fe5be5852ac404e91e1caca1ad89cb77b5da&amp;goto=news" class="clicky hider">hide</a> | <a href="item?id=43299772">16&nbsp;comments</a> </span>
                            </td>
                        </tr>
                        <tr class="spacer" style="height:5px"></tr>
                        <tr class='athing submission' id='43294566'>
                            <td align="right" valign="top" class="title"><span class="rank">16.</span></td>
                            <td valign="top" class="votelinks">
                                <center>
                                    <a id='up_43294566' class='clicky' href='vote?id=43294566&amp;how=up&amp;auth=487f45602988d3bcf2ec8c865a58bdad44eeb8bd&amp;goto=news'>
                                        <div class='votearrow' title='upvote'></div>
                                    </a>
                                </center>
                            </td>
                            <td class="title"><span class="titleline"><a href="https://pola.rs/posts/polars-cloud-what-we-are-building/">Polars Cloud: The Distributed Cloud Architecture to Run Polars Anywhere</a><span class="sitebit comhead"> (<a href="from?site=pola.rs"><span class="sitestr">pola.rs</span></a>)</span>
                                </span>
                            </td>
                        </tr>
                        <tr>
                            <td colspan="2"></td>
                            <td class="subtext"><span class="subline">
          <span class="score" id="score_43294566">226 points</span> by <a href="user?id=neilfrndes" class="hnuser">neilfrndes</a> <span class="age" title="2025-03-07T20:57:46 1741381066"><a href="item?id=43294566">18 hours ago</a></span> <span id="unv_43294566"></span> | <a href="flag?id=43294566&amp;auth=487f45602988d3bcf2ec8c865a58bdad44eeb8bd&amp;goto=news" rel="nofollow">flag</a> | <a href="hide?id=43294566&amp;auth=487f45602988d3bcf2ec8c865a58bdad44eeb8bd&amp;goto=news" class="clicky hider">hide</a> | <a href="item?id=43294566">82&nbsp;comments</a> </span>
                            </td>
                        </tr>
                        <tr class="spacer" style="height:5px"></tr>
                        <tr class='athing submission' id='43291922'>
                            <td align="right" valign="top" class="title"><span class="rank">17.</span></td>
                            <td valign="top" class="votelinks">
                                <center>
                                    <a id='up_43291922' class='clicky' href='vote?id=43291922&amp;how=up&amp;auth=dbb415af2a80940b9810efa932e39d7bd5c1ccc9&amp;goto=news'>
                                        <div class='votearrow' title='upvote'></div>
                                    </a>
                                </center>
                            </td>
                            <td class="title"><span class="titleline"><a href="item?id=43291922">Ask HN: Do your eyes bug you even though your prescription is &quot;correct&quot;?</a></span></td>
                        </tr>
                        <tr>
                            <td colspan="2"></td>
                            <td class="subtext"><span class="subline">
          <span class="score" id="score_43291922">272 points</span> by <a href="user?id=jbornhorst" class="hnuser">jbornhorst</a> <span class="age" title="2025-03-07T17:09:20 1741367360"><a href="item?id=43291922">22 hours ago</a></span> <span id="unv_43291922"></span> | <a href="flag?id=43291922&amp;auth=dbb415af2a80940b9810efa932e39d7bd5c1ccc9&amp;goto=news" rel="nofollow">flag</a> | <a href="hide?id=43291922&amp;auth=dbb415af2a80940b9810efa932e39d7bd5c1ccc9&amp;goto=news" class="clicky hider">hide</a> | <a href="item?id=43291922">256&nbsp;comments</a> </span>
                            </td>
                        </tr>
                        <tr class="spacer" style="height:5px"></tr>
                        <tr class='athing submission' id='43291946'>
                            <td align="right" valign="top" class="title"><span class="rank">18.</span></td>
                            <td valign="top" class="votelinks">
                                <center>
                                    <a id='up_43291946' class='clicky' href='vote?id=43291946&amp;how=up&amp;auth=3b29b4d3f6aad258f225a10b0339be265943349c&amp;goto=news'>
                                        <div class='votearrow' title='upvote'></div>
                                    </a>
                                </center>
                            </td>
                            <td class="title"><span class="titleline"><a href="https://github.com/directvt/vtm">Vtm: Text-Based Desktop Environment</a><span class="sitebit comhead"> (<a href="from?site=github.com/directvt"><span class="sitestr">github.com/directvt</span></a>)</span>
                                </span>
                            </td>
                        </tr>
                        <tr>
                            <td colspan="2"></td>
                            <td class="subtext"><span class="subline">
          <span class="score" id="score_43291946">262 points</span> by <a href="user?id=klaussilveira" class="hnuser">klaussilveira</a> <span class="age" title="2025-03-07T17:12:30 1741367550"><a href="item?id=43291946">22 hours ago</a></span> <span id="unv_43291946"></span> | <a href="flag?id=43291946&amp;auth=3b29b4d3f6aad258f225a10b0339be265943349c&amp;goto=news" rel="nofollow">flag</a> | <a href="hide?id=43291946&amp;auth=3b29b4d3f6aad258f225a10b0339be265943349c&amp;goto=news" class="clicky hider">hide</a> | <a href="item?id=43291946">76&nbsp;comments</a> </span>
                            </td>
                        </tr>
                        <tr class="spacer" style="height:5px"></tr>
                        <tr class='athing submission' id='43299635'>
                            <td align="right" valign="top" class="title"><span class="rank">19.</span></td>
                            <td valign="top" class="votelinks">
                                <center>
                                    <a id='up_43299635' class='clicky' href='vote?id=43299635&amp;how=up&amp;auth=8025cc5e93db8c431a360c1ca94d81e6acbd7bf4&amp;goto=news'>
                                        <div class='votearrow' title='upvote'></div>
                                    </a>
                                </center>
                            </td>
                            <td class="title"><span class="titleline"><a href="https://www.bloomberg.com/news/articles/2025-03-07/the-case-for-ditching-digital-memories-for-physical-objects">What We Lose When Our Memories Exist in Our Phones</a><span class="sitebit comhead"> (<a href="from?site=bloomberg.com"><span class="sitestr">bloomberg.com</span></a>)</span>
                                </span>
                            </td>
                        </tr>
                        <tr>
                            <td colspan="2"></td>
                            <td class="subtext"><span class="subline">
          <span class="score" id="score_43299635">36 points</span> by <a href="user?id=JumpCrisscross" class="hnuser">JumpCrisscross</a> <span class="age" title="2025-03-08T12:19:59 1741436399"><a href="item?id=43299635">3 hours ago</a></span> <span id="unv_43299635"></span> | <a href="flag?id=43299635&amp;auth=8025cc5e93db8c431a360c1ca94d81e6acbd7bf4&amp;goto=news" rel="nofollow">flag</a> | <a href="hide?id=43299635&amp;auth=8025cc5e93db8c431a360c1ca94d81e6acbd7bf4&amp;goto=news" class="clicky hider">hide</a> | <a href="item?id=43299635">12&nbsp;comments</a> </span>
                            </td>
                        </tr>
                        <tr class="spacer" style="height:5px"></tr>
                        <tr class='athing submission' id='43294974'>
                            <td align="right" valign="top" class="title"><span class="rank">20.</span></td>
                            <td valign="top" class="votelinks">
                                <center>
                                    <a id='up_43294974' class='clicky' href='vote?id=43294974&amp;how=up&amp;auth=4058331300a8ebaa55f9f0d75770e4bcbf581943&amp;goto=news'>
                                        <div class='votearrow' title='upvote'></div>
                                    </a>
                                </center>
                            </td>
                            <td class="title"><span class="titleline"><a href="https://github.com/letta-ai/letta">Letta: Letta is a framework for creating LLM services with memory</a><span class="sitebit comhead"> (<a href="from?site=github.com/letta-ai"><span class="sitestr">github.com/letta-ai</span></a>)</span>
                                </span>
                            </td>
                        </tr>
                        <tr>
                            <td colspan="2"></td>
                            <td class="subtext"><span class="subline">
          <span class="score" id="score_43294974">85 points</span> by <a href="user?id=sebg" class="hnuser">sebg</a> <span class="age" title="2025-03-07T21:33:43 1741383223"><a href="item?id=43294974">18 hours ago</a></span> <span id="unv_43294974"></span> | <a href="flag?id=43294974&amp;auth=4058331300a8ebaa55f9f0d75770e4bcbf581943&amp;goto=news" rel="nofollow">flag</a> | <a href="hide?id=43294974&amp;auth=4058331300a8ebaa55f9f0d75770e4bcbf581943&amp;goto=news" class="clicky hider">hide</a> | <a href="item?id=43294974">11&nbsp;comments</a> </span>
                            </td>
                        </tr>
                        <tr class="spacer" style="height:5px"></tr>
                        <tr class='athing submission' id='43296656'>
                            <td align="right" valign="top" class="title"><span class="rank">21.</span></td>
                            <td valign="top" class="votelinks">
                                <center>
                                    <a id='up_43296656' class='clicky' href='vote?id=43296656&amp;how=up&amp;auth=58edf87ab900b8b247eeb438bde388ad2e9fcfe0&amp;goto=news'>
                                        <div class='votearrow' title='upvote'></div>
                                    </a>
                                </center>
                            </td>
                            <td class="title"><span class="titleline"><a href="https://krebsonsecurity.com/2025/03/feds-link-150m-cyberheist-to-2022-lastpass-hacks/">Feds Link Cyberheist to 2022 LastPass Hacks</a><span class="sitebit comhead"> (<a href="from?site=krebsonsecurity.com"><span class="sitestr">krebsonsecurity.com</span></a>)</span>
                                </span>
                            </td>
                        </tr>
                        <tr>
                            <td colspan="2"></td>
                            <td class="subtext"><span class="subline">
          <span class="score" id="score_43296656">350 points</span> by <a href="user?id=todsacerdoti" class="hnuser">todsacerdoti</a> <span class="age" title="2025-03-08T01:26:33 1741397193"><a href="item?id=43296656">14 hours ago</a></span> <span id="unv_43296656"></span> | <a href="flag?id=43296656&amp;auth=58edf87ab900b8b247eeb438bde388ad2e9fcfe0&amp;goto=news" rel="nofollow">flag</a> | <a href="hide?id=43296656&amp;auth=58edf87ab900b8b247eeb438bde388ad2e9fcfe0&amp;goto=news" class="clicky hider">hide</a> | <a href="item?id=43296656">226&nbsp;comments</a> </span>
                            </td>
                        </tr>
                        <tr class="spacer" style="height:5px"></tr>
                        <tr class='athing submission' id='43256802'>
                            <td align="right" valign="top" class="title"><span class="rank">22.</span></td>
                            <td valign="top" class="votelinks">
                                <center>
                                    <a id='up_43256802' class='clicky' href='vote?id=43256802&amp;how=up&amp;auth=ed05ec76eacd0485e0f940e15e57250c76007d29&amp;goto=news'>
                                        <div class='votearrow' title='upvote'></div>
                                    </a>
                                </center>
                            </td>
                            <td class="title"><span class="titleline"><a href="https://rewiring.bearblog.dev/azure-devops-in-action-pt-iii-reasonably-secure-deploys-to-iis/">(Reasonably) secure Azure Pipelines on-prem deployments</a><span class="sitebit comhead"> (<a href="from?site=rewiring.bearblog.dev"><span class="sitestr">rewiring.bearblog.dev</span></a>)</span>
                                </span>
                            </td>
                        </tr>
                        <tr>
                            <td colspan="2"></td>
                            <td class="subtext"><span class="subline">
          <span class="score" id="score_43256802">26 points</span> by <a href="user?id=Mossy9" class="hnuser">Mossy9</a> <span class="age" title="2025-03-04T16:25:54 1741105554"><a href="item?id=43256802">11 hours ago</a></span> <span id="unv_43256802"></span> | <a href="flag?id=43256802&amp;auth=ed05ec76eacd0485e0f940e15e57250c76007d29&amp;goto=news" rel="nofollow">flag</a> | <a href="hide?id=43256802&amp;auth=ed05ec76eacd0485e0f940e15e57250c76007d29&amp;goto=news" class="clicky hider">hide</a> | <a href="item?id=43256802">31&nbsp;comments</a> </span>
                            </td>
                        </tr>
                        <tr class="spacer" style="height:5px"></tr>
                        <tr class='athing submission' id='43292050'>
                            <td align="right" valign="top" class="title"><span class="rank">23.</span></td>
                            <td valign="top" class="votelinks">
                                <center>
                                    <a id='up_43292050' class='clicky' href='vote?id=43292050&amp;how=up&amp;auth=6e245a1c5cffe39579a88ccc65aca48fb7a726ef&amp;goto=news'>
                                        <div class='votearrow' title='upvote'></div>
                                    </a>
                                </center>
                            </td>
                            <td class="title"><span class="titleline"><a href="https://cedardb.com/blog/optimistic_btrees/">Optimistic Locking in B-Trees</a><span class="sitebit comhead"> (<a href="from?site=cedardb.com"><span class="sitestr">cedardb.com</span></a>)</span>
                                </span>
                            </td>
                        </tr>
                        <tr>
                            <td colspan="2"></td>
                            <td class="subtext"><span class="subline">
          <span class="score" id="score_43292050">159 points</span> by <a href="user?id=uds5501" class="hnuser">uds5501</a> <span class="age" title="2025-03-07T17:23:28 1741368208"><a href="item?id=43292050">22 hours ago</a></span> <span id="unv_43292050"></span> | <a href="flag?id=43292050&amp;auth=6e245a1c5cffe39579a88ccc65aca48fb7a726ef&amp;goto=news" rel="nofollow">flag</a> | <a href="hide?id=43292050&amp;auth=6e245a1c5cffe39579a88ccc65aca48fb7a726ef&amp;goto=news" class="clicky hider">hide</a> | <a href="item?id=43292050">44&nbsp;comments</a> </span>
                            </td>
                        </tr>
                        <tr class="spacer" style="height:5px"></tr>
                        <tr class='athing submission' id='43292820'>
                            <td align="right" valign="top" class="title"><span class="rank">24.</span></td>
                            <td valign="top" class="votelinks">
                                <center>
                                    <a id='up_43292820' class='clicky' href='vote?id=43292820&amp;how=up&amp;auth=c538f6b1cfef7e7f58e6a4d308f31a68950a02aa&amp;goto=news'>
                                        <div class='votearrow' title='upvote'></div>
                                    </a>
                                </center>
                            </td>
                            <td class="title"><span class="titleline"><a href="https://www.eff.org/deeplinks/2025/03/first-porn-now-skin-cream-age-verification-bills-are-out-control">Age Verification Laws: A Backdoor to Surveillance</a><span class="sitebit comhead"> (<a href="from?site=eff.org"><span class="sitestr">eff.org</span></a>)</span>
                                </span>
                            </td>
                        </tr>
                        <tr>
                            <td colspan="2"></td>
                            <td class="subtext"><span class="subline">
          <span class="score" id="score_43292820">532 points</span> by <a href="user?id=hn_acker" class="hnuser">hn_acker</a> <span class="age" title="2025-03-07T18:34:02 1741372442"><a href="item?id=43292820">21 hours ago</a></span> <span id="unv_43292820"></span> | <a href="flag?id=43292820&amp;auth=c538f6b1cfef7e7f58e6a4d308f31a68950a02aa&amp;goto=news" rel="nofollow">flag</a> | <a href="hide?id=43292820&amp;auth=c538f6b1cfef7e7f58e6a4d308f31a68950a02aa&amp;goto=news" class="clicky hider">hide</a> | <a href="item?id=43292820">345&nbsp;comments</a> </span>
                            </td>
                        </tr>
                        <tr class="spacer" style="height:5px"></tr>
                        <tr class='athing submission' id='43266546'>
                            <td align="right" valign="top" class="title"><span class="rank">25.</span></td>
                            <td valign="top" class="votelinks">
                                <center>
                                    <a id='up_43266546' class='clicky' href='vote?id=43266546&amp;how=up&amp;auth=c1fa0ad872eaed250040dd169b5c42c9bc7d7446&amp;goto=news'>
                                        <div class='votearrow' title='upvote'></div>
                                    </a>
                                </center>
                            </td>
                            <td class="title"><span class="titleline"><a href="https://www.pythonmorsels.com/help-features/">The features of Python&#x27;s help() function</a><span class="sitebit comhead"> (<a href="from?site=pythonmorsels.com"><span class="sitestr">pythonmorsels.com</span></a>)</span>
                                </span>
                            </td>
                        </tr>
                        <tr>
                            <td colspan="2"></td>
                            <td class="subtext"><span class="subline">
          <span class="score" id="score_43266546">127 points</span> by <a href="user?id=danso" class="hnuser">danso</a> <span class="age" title="2025-03-05T14:07:24 1741183644"><a href="item?id=43266546">21 hours ago</a></span> <span id="unv_43266546"></span> | <a href="flag?id=43266546&amp;auth=c1fa0ad872eaed250040dd169b5c42c9bc7d7446&amp;goto=news" rel="nofollow">flag</a> | <a href="hide?id=43266546&amp;auth=c1fa0ad872eaed250040dd169b5c42c9bc7d7446&amp;goto=news" class="clicky hider">hide</a> | <a href="item?id=43266546">52&nbsp;comments</a> </span>
                            </td>
                        </tr>
                        <tr class="spacer" style="height:5px"></tr>
                        <tr class='athing submission' id='43292471'>
                            <td align="right" valign="top" class="title"><span class="rank">26.</span></td>
                            <td valign="top" class="votelinks">
                                <center>
                                    <a id='up_43292471' class='clicky' href='vote?id=43292471&amp;how=up&amp;auth=0d6a4e0e299c235137492410361dc29df73ff4d0&amp;goto=news'>
                                        <div class='votearrow' title='upvote'></div>
                                    </a>
                                </center>
                            </td>
                            <td class="title"><span class="titleline"><a href="https://www.theguardian.com/science/2025/mar/07/athena-spacecraft-mission-dead">Athena spacecraft declared dead after toppling over on moon</a><span class="sitebit comhead"> (<a href="from?site=theguardian.com"><span class="sitestr">theguardian.com</span></a>)</span>
                                </span>
                            </td>
                        </tr>
                        <tr>
                            <td colspan="2"></td>
                            <td class="subtext"><span class="subline">
          <span class="score" id="score_43292471">229 points</span> by <a href="user?id=pseudolus" class="hnuser">pseudolus</a> <span class="age" title="2025-03-07T18:06:45 1741370805"><a href="item?id=43292471">21 hours ago</a></span> <span id="unv_43292471"></span> | <a href="flag?id=43292471&amp;auth=0d6a4e0e299c235137492410361dc29df73ff4d0&amp;goto=news" rel="nofollow">flag</a> | <a href="hide?id=43292471&amp;auth=0d6a4e0e299c235137492410361dc29df73ff4d0&amp;goto=news" class="clicky hider">hide</a> | <a href="item?id=43292471">298&nbsp;comments</a> </span>
                            </td>
                        </tr>
                        <tr class="spacer" style="height:5px"></tr>
                        <tr class='athing submission' id='43295865'>
                            <td align="right" valign="top" class="title"><span class="rank">27.</span></td>
                            <td valign="top" class="votelinks">
                                <center>
                                    <a id='up_43295865' class='clicky' href='vote?id=43295865&amp;how=up&amp;auth=3e4a362da3dce0244c1c8cfdc23c7272d68924ae&amp;goto=news'>
                                        <div class='votearrow' title='upvote'></div>
                                    </a>
                                </center>
                            </td>
                            <td class="title"><span class="titleline"><a href="https://www.freaktakes.com/p/how-did-places-like-bell-labs-know">How did places like Bell Labs know how to ask the right questions? (2023)</a><span class="sitebit comhead"> (<a href="from?site=freaktakes.com"><span class="sitestr">freaktakes.com</span></a>)</span>
                                </span>
                            </td>
                        </tr>
                        <tr>
                            <td colspan="2"></td>
                            <td class="subtext"><span class="subline">
          <span class="score" id="score_43295865">140 points</span> by <a href="user?id=sebg" class="hnuser">sebg</a> <span class="age" title="2025-03-07T23:14:36 1741389276"><a href="item?id=43295865">16 hours ago</a></span> <span id="unv_43295865"></span> | <a href="flag?id=43295865&amp;auth=3e4a362da3dce0244c1c8cfdc23c7272d68924ae&amp;goto=news" rel="nofollow">flag</a> | <a href="hide?id=43295865&amp;auth=3e4a362da3dce0244c1c8cfdc23c7272d68924ae&amp;goto=news" class="clicky hider">hide</a> | <a href="item?id=43295865">90&nbsp;comments</a> </span>
                            </td>
                        </tr>
                        <tr class="spacer" style="height:5px"></tr>
                        <tr class='athing submission' id='43298408'>
                            <td align="right" valign="top" class="title"><span class="rank">28.</span></td>
                            <td valign="top" class="votelinks">
                                <center>
                                    <a id='up_43298408' class='clicky' href='vote?id=43298408&amp;how=up&amp;auth=6e236a66b7188a6747021717029270f7f3b5d3d1&amp;goto=news'>
                                        <div class='votearrow' title='upvote'></div>
                                    </a>
                                </center>
                            </td>
                            <td class="title"><span class="titleline"><a href="https://manus.im/" rel="nofollow">Leave It to Manus</a><span class="sitebit comhead"> (<a href="from?site=manus.im"><span class="sitestr">manus.im</span></a>)</span>
                                </span>
                            </td>
                        </tr>
                        <tr>
                            <td colspan="2"></td>
                            <td class="subtext"><span class="subline">
          <span class="score" id="score_43298408">9 points</span> by <a href="user?id=doener" class="hnuser">doener</a> <span class="age" title="2025-03-08T07:56:50 1741420610"><a href="item?id=43298408">7 hours ago</a></span> <span id="unv_43298408"></span> | <a href="flag?id=43298408&amp;auth=6e236a66b7188a6747021717029270f7f3b5d3d1&amp;goto=news" rel="nofollow">flag</a> | <a href="hide?id=43298408&amp;auth=6e236a66b7188a6747021717029270f7f3b5d3d1&amp;goto=news" class="clicky hider">hide</a> | <a href="item?id=43298408">2&nbsp;comments</a> </span>
                            </td>
                        </tr>
                        <tr class="spacer" style="height:5px"></tr>
                        <tr class='athing submission' id='43290555'>
                            <td align="right" valign="top" class="title"><span class="rank">29.</span></td>
                            <td valign="top" class="votelinks">
                                <center>
                                    <a id='up_43290555' class='clicky' href='vote?id=43290555&amp;how=up&amp;auth=dc438363641f7e3bdec562f7fc3b9a0822147fa3&amp;goto=news'>
                                        <div class='votearrow' title='upvote'></div>
                                    </a>
                                </center>
                            </td>
                            <td class="title"><span class="titleline"><a href="https://engineering.fb.com/2025/01/21/production-engineering/strobelight-a-profiling-service-built-on-open-source-technology/">Strobelight: A profiling service built on open source technology</a><span class="sitebit comhead"> (<a href="from?site=fb.com"><span class="sitestr">fb.com</span></a>)</span>
                                </span>
                            </td>
                        </tr>
                        <tr>
                            <td colspan="2"></td>
                            <td class="subtext"><span class="subline">
          <span class="score" id="score_43290555">158 points</span> by <a href="user?id=birdculture" class="hnuser">birdculture</a> <span class="age" title="2025-03-07T14:43:24 1741358604"><a href="item?id=43290555">1 day ago</a></span> <span id="unv_43290555"></span> | <a href="flag?id=43290555&amp;auth=dc438363641f7e3bdec562f7fc3b9a0822147fa3&amp;goto=news" rel="nofollow">flag</a> | <a href="hide?id=43290555&amp;auth=dc438363641f7e3bdec562f7fc3b9a0822147fa3&amp;goto=news" class="clicky hider">hide</a> | <a href="item?id=43290555">47&nbsp;comments</a> </span>
                            </td>
                        </tr>
                        <tr class="spacer" style="height:5px"></tr>
                        <tr class='athing submission' id='43294489'>
                            <td align="right" valign="top" class="title"><span class="rank">30.</span></td>
                            <td valign="top" class="votelinks">
                                <center>
                                    <a id='up_43294489' class='clicky' href='vote?id=43294489&amp;how=up&amp;auth=ed97990d02b45c50d20356ae96eb2bf874a69a44&amp;goto=news'>
                                        <div class='votearrow' title='upvote'></div>
                                    </a>
                                </center>
                            </td>
                            <td class="title"><span class="titleline"><a href="https://www.quantamagazine.org/next-level-chaos-traces-the-true-limit-of-predictability-20250307/">&#x27;Next-Level&#x27; Chaos Traces the True Limit of Predictability</a><span class="sitebit comhead"> (<a href="from?site=quantamagazine.org"><span class="sitestr">quantamagazine.org</span></a>)</span>
                                </span>
                            </td>
                        </tr>
                        <tr>
                            <td colspan="2"></td>
                            <td class="subtext"><span class="subline">
          <span class="score" id="score_43294489">65 points</span> by <a href="user?id=pseudolus" class="hnuser">pseudolus</a> <span class="age" title="2025-03-07T20:50:45 1741380645"><a href="item?id=43294489">18 hours ago</a></span> <span id="unv_43294489"></span> | <a href="flag?id=43294489&amp;auth=ed97990d02b45c50d20356ae96eb2bf874a69a44&amp;goto=news" rel="nofollow">flag</a> | <a href="hide?id=43294489&amp;auth=ed97990d02b45c50d20356ae96eb2bf874a69a44&amp;goto=news" class="clicky hider">hide</a> | <a href="item?id=43294489">20&nbsp;comments</a> </span>
                            </td>
                        </tr>
                        <tr class="spacer" style="height:5px"></tr>
                        <tr class="morespace" style="height:10px"></tr>
                        <tr>
                            <td colspan="2"></td>
                            <td class='title'><a href='?p=2' class='morelink' rel='next'>More</a></td>
                        </tr>
                    </table>
                </td>
            </tr>
            <tr>
                <td><img src="s.gif" height="10" width="0">
                    <table width="100%" cellspacing="0" cellpadding="1">
                        <tr>
                            <td bgcolor="#ff6600"></td>
                        </tr>
                    </table><br>
                    <center>Join us for <a href="https://events.ycombinator.com/ai-sus"><u>AI Startup School</u></a> this June 16-17 in San Francisco!</center><br>
                    <center><span class="yclinks"><a href="newsguidelines.html">Guidelines</a> | <a href="newsfaq.html">FAQ</a> | <a href="lists">Lists</a> | <a href="https://github.com/HackerNews/API">API</a> | <a href="security.html">Security</a> | <a href="https://www.ycombinator.com/legal/">Legal</a> | <a href="https://www.ycombinator.com/apply/">Apply to YC</a> | <a href="mailto:hn@ycombinator.com">Contact</a></span><br><br>
                        <form method="get" action="//hn.algolia.com/">Search: <input type="text" name="q" size="17" autocorrect="off" spellcheck="false" autocapitalize="off" autocomplete="off"></form>
                    </center>
                </td>
            </tr>
        </table>
    </center>
</body>
<script type='text/javascript' src='hn.js?0Cm7vITOnsFHYgoWtXzU'></script>
</html>
"###;
