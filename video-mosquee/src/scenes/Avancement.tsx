import React from 'react';
import {AbsoluteFill, interpolate, spring, useCurrentFrame, useVideoConfig} from 'remotion';
import {CheckMark, Kicker, MaskLine, Rise, SceneExit} from '../components/Common';
import {C, formatNumber, pop, progress, sans, serif} from '../theme';

export const AVANCEMENT_DURATION = 420;

const LEFT = 112;
const COL_W = 397;
const GAP = 36;
const LINE_Y = 426;
const CIRCLE = 120;
const center = (i: number) => LEFT + i * (COL_W + GAP) + COL_W / 2;

// Timing (en frames, 30 i/s) de chaque étape de la frise.
const T = {
	step1: 40,
	line12: [68, 98] as const,
	step2: 98,
	line23: [126, 156] as const,
	step3: 156,
	step4: 205,
	counter: [175, 235] as const,
};

const stepLabel: React.CSSProperties = {
	fontFamily: sans,
	fontWeight: 700,
	fontSize: 31,
	letterSpacing: 1.5,
	color: C.label,
	textAlign: 'center',
};

const cardTitle: React.CSSProperties = {
	fontFamily: serif,
	fontWeight: 600,
	fontSize: 39,
	lineHeight: 1.18,
	color: C.green,
	textAlign: 'center',
};

const bigValue: React.CSSProperties = {
	fontFamily: sans,
	fontWeight: 800,
	fontSize: 56,
	lineHeight: 1.25,
	color: C.green,
	textAlign: 'center',
};

const smallText: React.CSSProperties = {
	fontFamily: sans,
	fontWeight: 400,
	fontSize: 29,
	color: C.text,
	textAlign: 'center',
};

const DoneCircle: React.FC<{i: number; delay: number}> = ({i, delay}) => {
	const frame = useCurrentFrame();
	const {fps} = useVideoConfig();
	const s = pop(frame, fps, delay);
	return (
		<div
			style={{
				position: 'absolute',
				left: center(i) - CIRCLE / 2,
				top: LINE_Y - CIRCLE / 2,
				width: CIRCLE,
				height: CIRCLE,
				borderRadius: '50%',
				background: C.green,
				border: `6px solid ${C.white}`,
				boxShadow: `0 0 0 5px ${C.green}`,
				display: 'flex',
				alignItems: 'center',
				justifyContent: 'center',
				transform: `scale(${s})`,
			}}
		>
			<CheckMark size={92} progress={progress(frame, delay + 8, delay + 22)} />
		</div>
	);
};

const CurrentCircle: React.FC<{delay: number}> = ({delay}) => {
	const frame = useCurrentFrame();
	const {fps} = useVideoConfig();
	const s = pop(frame, fps, delay);
	const t = Math.max(0, frame - delay - 15);
	const pulse = (Math.sin(t / 9) + 1) / 2; // 0 → 1 en continu
	const ring = (t % 45) / 45; // onde qui s'élargit
	const plusRot = interpolate(s, [0, 1], [-180, 0]);
	return (
		<>
			{/* onde */}
			<div
				style={{
					position: 'absolute',
					left: center(2) - CIRCLE / 2,
					top: LINE_Y - CIRCLE / 2,
					width: CIRCLE,
					height: CIRCLE,
					borderRadius: '50%',
					border: `4px solid ${C.gold}`,
					opacity: frame > delay + 15 ? (1 - ring) * 0.6 : 0,
					transform: `scale(${1 + ring * 0.9})`,
				}}
			/>
			{/* halo */}
			<div
				style={{
					position: 'absolute',
					left: center(2) - 75,
					top: LINE_Y - 75,
					width: 150,
					height: 150,
					borderRadius: '50%',
					background: 'rgba(212,146,42,0.22)',
					transform: `scale(${s * (1 + pulse * 0.1)})`,
				}}
			/>
			<div
				style={{
					position: 'absolute',
					left: center(2) - CIRCLE / 2,
					top: LINE_Y - CIRCLE / 2,
					width: CIRCLE,
					height: CIRCLE,
					borderRadius: '50%',
					background: C.gold,
					border: `5px solid ${C.white}`,
					boxShadow: `0 0 0 4px ${C.gold}`,
					display: 'flex',
					alignItems: 'center',
					justifyContent: 'center',
					transform: `scale(${s})`,
				}}
			>
				<svg width={60} height={60} viewBox="0 0 60 60" style={{transform: `rotate(${plusRot}deg)`}}>
					<path d="M30 8 V52 M8 30 H52" stroke={C.white} strokeWidth={8} strokeLinecap="round" />
				</svg>
			</div>
		</>
	);
};

const Card: React.FC<{
	i: number;
	delay: number;
	variant?: 'normal' | 'current' | 'next';
	children: React.ReactNode;
}> = ({i, delay, variant = 'normal', children}) => {
	const frame = useCurrentFrame();
	const {fps} = useVideoConfig();
	const s = spring({frame: frame - delay, fps, config: {damping: 16, stiffness: 110}});
	const styles: Record<string, React.CSSProperties> = {
		normal: {background: C.white, border: `2px solid ${C.border}`},
		current: {
			background: C.white,
			border: `4px solid ${C.gold}`,
			boxShadow: '0 18px 40px rgba(212,146,42,0.18)',
		},
		next: {background: 'transparent', border: `2px dashed #e2ded8`},
	};
	return (
		<div
			style={{
				position: 'absolute',
				left: center(i) - COL_W / 2,
				top: 576,
				width: COL_W,
				height: 268,
				borderRadius: 16,
				display: 'flex',
				flexDirection: 'column',
				alignItems: 'center',
				justifyContent: 'center',
				padding: '0 24px',
				opacity: Math.min(1, s * 1.4),
				transform: `translateY(${(1 - s) * 70}px) scale(${interpolate(s, [0, 1], [0.92, 1])})`,
				...styles[variant],
			}}
		>
			{children}
		</div>
	);
};

const Label: React.FC<{i: number; delay: number; color?: string; children: React.ReactNode}> = ({
	i,
	delay,
	color,
	children,
}) => (
	<div style={{position: 'absolute', left: center(i) - COL_W / 2, top: 512, width: COL_W}}>
		<Rise delay={delay} distance={20}>
			<div style={{...stepLabel, color: color ?? C.label}}>{children}</div>
		</Rise>
	</div>
);

export const Avancement: React.FC = () => {
	const frame = useCurrentFrame();
	const {fps} = useVideoConfig();

	const x1 = center(0);
	const x2 = center(1);
	const x3 = center(2);
	const x4 = center(3);
	const greyIn = progress(frame, 20, 60);
	const p12 = progress(frame, ...T.line12);
	const p23 = progress(frame, ...T.line23);
	const greenEnd = x1 + (x2 - x1) * p12 + (x3 - x2) * p23;

	const counter = interpolate(frame, T.counter, [0, 100000], {
		extrapolateLeft: 'clamp',
		extrapolateRight: 'clamp',
		easing: (t) => 1 - Math.pow(1 - t, 3),
	});
	const counterDone = frame >= T.counter[1];
	const bump = spring({frame: frame - T.counter[1], fps, config: {damping: 8, stiffness: 200}});

	const accorde = spring({frame: frame - (T.step2 + 18), fps, config: {damping: 9, stiffness: 160}});

	const s4 = spring({frame: frame - T.step4, fps, config: {damping: 200}});

	return (
		<SceneExit duration={AVANCEMENT_DURATION}>
			<AbsoluteFill>
				{/* En-tête */}
				<div style={{position: 'absolute', left: LEFT, top: 118}}>
					<Kicker text="AVANCEMENT DU PROJET" ar="ما شاء الله" />
				</div>
				<div style={{position: 'absolute', left: LEFT - 2, top: 182}}>
					<MaskLine delay={12}>
						<div
							style={{
								fontFamily: serif,
								fontWeight: 700,
								fontSize: 108,
								lineHeight: 1.1,
								color: C.green,
							}}
						>
							Notre mosquée avance
						</div>
					</MaskLine>
				</div>

				{/* Frise : ligne grise de fond */}
				<div
					style={{
						position: 'absolute',
						left: x1,
						top: LINE_Y - 4,
						width: (x4 - x1) * greyIn,
						height: 8,
						borderRadius: 4,
						background: '#e9e7e3',
					}}
				/>
				{/* Frise : progression verte */}
				<div
					style={{
						position: 'absolute',
						left: x1,
						top: LINE_Y - 4,
						width: greenEnd - x1,
						height: 8,
						borderRadius: 4,
						background: C.green,
					}}
				/>

				{/* Étape 1 */}
				<DoneCircle i={0} delay={T.step1} />
				<Label i={0} delay={T.step1 + 6}>
					JANVIER 2024
				</Label>
				<Card i={0} delay={T.step1 + 10}>
					<div style={cardTitle}>Achat du local</div>
					<div style={bigValue}>380&#8239;000&nbsp;€</div>
				</Card>

				{/* Étape 2 */}
				<DoneCircle i={1} delay={T.step2} />
				<Label i={1} delay={T.step2 + 6}>
					4 NOVEMBRE 2026
				</Label>
				<Card i={1} delay={T.step2 + 10}>
					<div style={cardTitle}>
						Permis de
						<br />
						construire
					</div>
					<div
						style={{
							...bigValue,
							opacity: Math.min(1, accorde * 2),
							transform: `scale(${interpolate(accorde, [0, 1], [1.8, 1])})`,
						}}
					>
						Accordé
					</div>
				</Card>

				{/* Étape 3 : aujourd'hui */}
				<CurrentCircle delay={T.step3} />
				<Label i={2} delay={T.step3 + 6} color={C.gold}>
					AUJOURD&apos;HUI
				</Label>
				<Card i={2} delay={T.step3 + 10} variant="current">
					<div style={cardTitle}>
						Collecte gros
						<br />
						œuvre
					</div>
					<div
						style={{
							...bigValue,
							color: C.gold,
							fontVariantNumeric: 'tabular-nums',
							transform: `scale(${counterDone ? 1 + (1 - bump) * 0.15 : 1})`,
						}}
					>
						{formatNumber(counter)}&nbsp;€
					</div>
					<div style={smallText}>déjà récoltés</div>
				</Card>

				{/* Étape 4 : prochaine étape */}
				<div
					style={{
						position: 'absolute',
						left: x4 - CIRCLE / 2,
						top: LINE_Y - CIRCLE / 2,
						width: CIRCLE,
						height: CIRCLE,
						borderRadius: '50%',
						background: C.white,
						border: `6px solid #ededed`,
						display: 'flex',
						alignItems: 'center',
						justifyContent: 'center',
						fontFamily: sans,
						fontWeight: 600,
						fontSize: 40,
						color: C.grey,
						opacity: s4,
						transform: `scale(${interpolate(s4, [0, 1], [0.6, 1])})`,
					}}
				>
					4
				</div>
				<Label i={3} delay={T.step4 + 6}>
					PROCHAINE ÉTAPE
				</Label>
				<Card i={3} delay={T.step4 + 10} variant="next">
					<div style={{...cardTitle, color: C.text}}>
						Travaux de gros
						<br />
						œuvre
					</div>
					<div style={{...smallText, marginTop: 10}}>Lancement du chantier</div>
				</Card>
			</AbsoluteFill>
		</SceneExit>
	);
};
