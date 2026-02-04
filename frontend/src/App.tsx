import { Routes, Route } from 'react-router-dom';
import { Layout } from './components/Layout';
import { Home } from './pages/Home';
import { Explorer } from './pages/Explorer';
import { MyChannels } from './pages/MyChannels';
import { Campaigns } from './pages/Campaigns';
import { Deals } from './pages/Deals';

function App() {
  return (
    <Routes>
      <Route path="/" element={<Layout />}>
        <Route index element={<Home />} />
        <Route path="explorer" element={<Explorer />} />
        <Route path="channels" element={<MyChannels />} />
        <Route path="campaigns" element={<Campaigns />} />
        <Route path="deals" element={<Deals />} />
      </Route>
    </Routes>
  );
}

export default App;
