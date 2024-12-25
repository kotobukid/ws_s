<script lang="ts">
import {defineComponent, h} from "vue";

// 変化できるコンポーネントは、現状ホワイトリストで明記する
import {ChatFront} from "#components";
import {TabSync} from "#components";

const componentsDict = new Map();
componentsDict.set('ChatFront', ChatFront);
componentsDict.set('TabSync', TabSync);

export default defineComponent({
  props: {
    component: {
      type: String,
      required: true,
    },
  },
  setup(props) {
    return () => {
      const c = componentsDict.get(props.component) || undefined;

      if (c === undefined) {
        return h('span', ['no components found with name: ', props.component])
      } else {
        return h(c);
      }
    };
  },
});
</script>