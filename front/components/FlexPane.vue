<script lang="ts">
import {defineComponent, h} from "vue";
import {ComponentAppList} from "#components";

// 変化できるコンポーネントは、現状ホワイトリストで明記する
import {ChatFront} from "#components";
import {TabSync} from "#components";
import {SseReceiver} from "#components";
import type {ComponentAppType} from "~/types";

const componentsDict = new Map<ComponentAppType, typeof ChatFront | typeof TabSync>();
componentsDict.set('ChatFront', ChatFront);
componentsDict.set('TabSync', TabSync);
componentsDict.set('SseReceiver', SseReceiver);

export default defineComponent({
  props: {
    component: {
      type: String as PropType<ComponentAppType>,
      required: true,
    },
  },
  emits: {
    'switch-component': (component: ComponentAppType) => true,
  },
  data: () => {
    return {
      show_app_list: false,
    };
  },
  methods: {
    toggle_app_list() {
      this.show_app_list = !this.show_app_list;
    },
  },
  setup(props) {
    return function () {
      // @ts-ignore
      const that = this;
      const c = componentsDict.get(props.component) || undefined;

      if (c === undefined) {
        return h('div', [
          h('h1', [
            that.show_app_list ? h(ComponentAppList, {
              current: props.component,
              onSwitchComponent: (component: ComponentAppType) => {
                that.$emit('switch-component', component);
                that.show_app_list = false;
              },
            }) : null,
            h('a', {
              'class': 'toggle',
              href: "#",
              onClick(e: MouseEvent) {
                e.preventDefault();
                that.toggle_app_list();
              },
            }, ['＠']),
            h('span', [props.component])
          ]),
          h('span', ['no components found with name: ', props.component])])
      } else {
        return h('div', [
          h('h1', [
            that.show_app_list ? h(ComponentAppList, {
              current: props.component,
              onSwitchComponent: (component: ComponentAppType) => {
                that.$emit('switch-component', component);
                that.show_app_list = false;
              },
            }) : null,
            h('a', {
              href: "#",
              'class': 'toggle',
              onClick(e: MouseEvent) {
                e.preventDefault();
                that.toggle_app_list();
              },
            }, ['＠']),
            h('span', [props.component])
          ]),
          h(c)
        ]);
      }
    };
  },
});
</script>

<style scoped>
h1 {
  font-size: 10px;
  background-color: #6f7dd5;
  padding: 4px;
  margin: 0;
}

a.toggle {
  background-color: gold;
  color: black;
  text-decoration: none;
  cursor: pointer;
  border-radius: 3px;
  display: inline-block;
  width: 1.2rem;
  text-align: center;
  margin-right: 4px;
}
</style>