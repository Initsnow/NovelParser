import { BrowserRouter, Routes, Route } from 'react-router-dom';
import AppLayout from './components/layout/AppLayout';
import HomePage from './pages/HomePage';
import NovelPage from './pages/NovelPage';
import SummaryPage from './pages/SummaryPage';

export default function App() {
  return (
    <BrowserRouter>
      <Routes>
        <Route element={<AppLayout />}>
          <Route path="/" element={<HomePage />} />
          <Route path="/novel/:novelId" element={<NovelPage />} />
          <Route path="/novel/:novelId/summary" element={<SummaryPage />} />
        </Route>
      </Routes>
    </BrowserRouter>
  );
}
