const ImportCanvas = require('./components/Import/ImportCanvas');
const ImportPanel = require('./components/Import/ImportPanel');
const importSlice = require('./components/Import/slice');
const ModelList = require('./components/Import/ModelList');
const { configureStore } = require('@reduxjs/toolkit');

const store = configureStore({
  reducer: {
    import: importSlice.reducer
  },
});

let components = [
  new ImportCanvas(document.querySelector('#spritec-canvas'), store),
  new ImportPanel(document.querySelector('#spritec-import'), store),
  new ModelList(document.querySelector('#spritec-model-list'), store),
];

store.subscribe(() => {
  let state = store.getState();
  components.forEach((component) => {
    if (component._updateComponent(state)) {
      component.render();
    }
  });
});
