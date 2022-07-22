(function() {var implementors = {};
implementors["gear_common"] = [{"text":"impl&lt;Key, Value, Error, HVS, TVS, MS, Callbacks&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html\" title=\"trait core::iter::traits::iterator::Iterator\">Iterator</a> for <a class=\"struct\" href=\"gear_common/storage/struct.DequeueDrainIter.html\" title=\"struct gear_common::storage::DequeueDrainIter\">DequeueDrainIter</a>&lt;Key, Value, Error, HVS, TVS, MS, Callbacks&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Key: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;Error: <a class=\"trait\" href=\"gear_common/storage/trait.DequeueError.html\" title=\"trait gear_common::storage::DequeueError\">DequeueError</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;HVS: <a class=\"trait\" href=\"gear_common/storage/trait.ValueStorage.html\" title=\"trait gear_common::storage::ValueStorage\">ValueStorage</a>&lt;Value = Key&gt;,<br>&nbsp;&nbsp;&nbsp;&nbsp;TVS: <a class=\"trait\" href=\"gear_common/storage/trait.ValueStorage.html\" title=\"trait gear_common::storage::ValueStorage\">ValueStorage</a>&lt;Value = Key&gt;,<br>&nbsp;&nbsp;&nbsp;&nbsp;MS: <a class=\"trait\" href=\"gear_common/storage/trait.MapStorage.html\" title=\"trait gear_common::storage::MapStorage\">MapStorage</a>&lt;Key = Key, Value = <a class=\"struct\" href=\"gear_common/storage/struct.LinkedNode.html\" title=\"struct gear_common::storage::LinkedNode\">LinkedNode</a>&lt;Key, Value&gt;&gt;,<br>&nbsp;&nbsp;&nbsp;&nbsp;Callbacks: <a class=\"trait\" href=\"gear_common/storage/trait.DequeueCallbacks.html\" title=\"trait gear_common::storage::DequeueCallbacks\">DequeueCallbacks</a>&lt;Value = Value&gt;,&nbsp;</span>","synthetic":false,"types":["gear_common::storage::complicated::dequeue::DequeueDrainIter"]},{"text":"impl&lt;Key, Value, Error, HVS, TVS, MS&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html\" title=\"trait core::iter::traits::iterator::Iterator\">Iterator</a> for <a class=\"struct\" href=\"gear_common/storage/struct.DequeueIter.html\" title=\"struct gear_common::storage::DequeueIter\">DequeueIter</a>&lt;Key, Value, Error, HVS, TVS, MS&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;Key: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/cmp/trait.PartialEq.html\" title=\"trait core::cmp::PartialEq\">PartialEq</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;Error: <a class=\"trait\" href=\"gear_common/storage/trait.DequeueError.html\" title=\"trait gear_common::storage::DequeueError\">DequeueError</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;HVS: <a class=\"trait\" href=\"gear_common/storage/trait.ValueStorage.html\" title=\"trait gear_common::storage::ValueStorage\">ValueStorage</a>&lt;Value = Key&gt;,<br>&nbsp;&nbsp;&nbsp;&nbsp;TVS: <a class=\"trait\" href=\"gear_common/storage/trait.ValueStorage.html\" title=\"trait gear_common::storage::ValueStorage\">ValueStorage</a>&lt;Value = Key&gt;,<br>&nbsp;&nbsp;&nbsp;&nbsp;MS: <a class=\"trait\" href=\"gear_common/storage/trait.MapStorage.html\" title=\"trait gear_common::storage::MapStorage\">MapStorage</a>&lt;Key = Key, Value = <a class=\"struct\" href=\"gear_common/storage/struct.LinkedNode.html\" title=\"struct gear_common::storage::LinkedNode\">LinkedNode</a>&lt;Key, Value&gt;&gt;,&nbsp;</span>","synthetic":false,"types":["gear_common::storage::complicated::dequeue::DequeueIter"]},{"text":"impl&lt;I, Item, TC&gt; <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html\" title=\"trait core::iter::traits::iterator::Iterator\">Iterator</a> for <a class=\"struct\" href=\"gear_common/storage/struct.IteratorWrap.html\" title=\"struct gear_common::storage::IteratorWrap\">IteratorWrap</a>&lt;I, Item, TC&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;I: <a class=\"trait\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html\" title=\"trait core::iter::traits::iterator::Iterator\">Iterator</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;TC: <a class=\"trait\" href=\"gear_common/storage/trait.TransposeCallback.html\" title=\"trait gear_common::storage::TransposeCallback\">TransposeCallback</a>&lt;I::<a class=\"associatedtype\" href=\"https://doc.rust-lang.org/nightly/core/iter/traits/iterator/trait.Iterator.html#associatedtype.Item\" title=\"type core::iter::traits::iterator::Iterator::Item\">Item</a>, Item&gt;,&nbsp;</span>","synthetic":false,"types":["gear_common::storage::primitives::iterable::IteratorWrap"]}];
implementors["gstd"] = [];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()