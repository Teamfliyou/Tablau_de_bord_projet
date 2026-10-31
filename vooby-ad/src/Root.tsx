import React from 'react';
import {Composition} from 'remotion';
import {VoobyAd} from './VoobyAd';
import {FPS, TOTAL_FRAMES} from './constants';
import './fonts';

export const Root: React.FC = () => (
	<Composition
		id="VoobyBlackNovember"
		component={VoobyAd}
		durationInFrames={TOTAL_FRAMES}
		fps={FPS}
		width={1080}
		height={1920}
	/>
);
